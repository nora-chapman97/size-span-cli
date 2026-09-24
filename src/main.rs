use sizespan::bytes::ByteSize;
use sizespan::duration::{format_duration, parse_duration};
use std::env;
use std::process::ExitCode;
use std::str::FromStr;
use std::time::Duration;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let (json, args) = strip_json_flag(args);
    match args {
        [cmd, sub, value] if cmd == "bytes" && sub == "parse" => {
            let size = ByteSize::from_str(value).map_err(|e| e.to_string())?;
            if json {
                Ok(json_object(&[
                    ("input", JsonValue::Str(value)),
                    ("bytes", JsonValue::Num(size.as_bytes() as f64)),
                ]))
            } else {
                Ok(format!("{} bytes", size.as_bytes()))
            }
        }
        [cmd, sub, value] if cmd == "bytes" && sub == "fmt" => {
            let n: u64 = value
                .parse()
                .map_err(|_| format!("invalid byte count: {value:?}"))?;
            let text = ByteSize::from_bytes(n).to_string();
            if json {
                Ok(json_object(&[
                    ("input", JsonValue::Str(value)),
                    ("text", JsonValue::Str(&text)),
                ]))
            } else {
                Ok(text)
            }
        }
        [cmd, sub, value] if cmd == "duration" && sub == "parse" => {
            let d = parse_duration(value).map_err(|e| e.to_string())?;
            if json {
                Ok(json_object(&[
                    ("input", JsonValue::Str(value)),
                    ("ms", JsonValue::Num(d.as_millis() as f64)),
                ]))
            } else {
                Ok(format!("{} ms", d.as_millis()))
            }
        }
        [cmd, sub, value] if cmd == "duration" && sub == "fmt" => {
            let secs: f64 = value
                .parse()
                .map_err(|_| format!("invalid seconds: {value:?}"))?;
            if secs < 0.0 || !secs.is_finite() {
                return Err("duration cannot be negative".to_string());
            }
            let text = format_duration(Duration::from_secs_f64(secs));
            if json {
                Ok(json_object(&[
                    ("input", JsonValue::Str(value)),
                    ("text", JsonValue::Str(&text)),
                ]))
            } else {
                Ok(text)
            }
        }
        _ => Err(usage()),
    }
}

/// Pulls a trailing `--json` flag off the argument list, if present.
///
/// Only checked at the end since every subcommand here takes a fixed
/// number of positional arguments - there's no getopt-style parsing to
/// worry about yet.
fn strip_json_flag(args: &[String]) -> (bool, &[String]) {
    match args.split_last() {
        Some((last, rest)) if last == "--json" => (true, rest),
        _ => (false, args),
    }
}

enum JsonValue<'a> {
    Str(&'a str),
    Num(f64),
}

fn json_object(fields: &[(&str, JsonValue)]) -> String {
    let mut out = String::from("{");
    for (i, (key, value)) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(key);
        out.push_str("\":");
        match value {
            JsonValue::Str(s) => {
                out.push('"');
                out.push_str(&json_escape(s));
                out.push('"');
            }
            JsonValue::Num(n) => out.push_str(&n.to_string()),
        }
    }
    out.push('}');
    out
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn usage() -> String {
    "usage:\n  \
     sizespan bytes parse <text> [--json]      e.g. sizespan bytes parse 1.5MiB\n  \
     sizespan bytes fmt <n> [--json]           e.g. sizespan bytes fmt 1572864\n  \
     sizespan duration parse <text> [--json]   e.g. sizespan duration parse 1h30m\n  \
     sizespan duration fmt <seconds> [--json]  e.g. sizespan duration fmt 5400"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_output_is_unchanged() {
        assert_eq!(
            run(&["bytes".into(), "parse".into(), "1.5MiB".into()]).unwrap(),
            "1572864 bytes"
        );
        assert_eq!(
            run(&["duration".into(), "fmt".into(), "5400".into()]).unwrap(),
            "1h30m"
        );
    }

    #[test]
    fn json_output_for_each_subcommand() {
        assert_eq!(
            run(&[
                "bytes".into(),
                "parse".into(),
                "1.5MiB".into(),
                "--json".into()
            ])
            .unwrap(),
            r#"{"input":"1.5MiB","bytes":1572864}"#
        );
        assert_eq!(
            run(&[
                "bytes".into(),
                "fmt".into(),
                "1572864".into(),
                "--json".into()
            ])
            .unwrap(),
            r#"{"input":"1572864","text":"1.5 MiB"}"#
        );
        assert_eq!(
            run(&[
                "duration".into(),
                "parse".into(),
                "1h30m".into(),
                "--json".into()
            ])
            .unwrap(),
            r#"{"input":"1h30m","ms":5400000}"#
        );
        assert_eq!(
            run(&[
                "duration".into(),
                "fmt".into(),
                "5400".into(),
                "--json".into()
            ])
            .unwrap(),
            r#"{"input":"5400","text":"1h30m"}"#
        );
    }

    #[test]
    fn json_errors_still_pass_through_as_plain_text() {
        let err = run(&[
            "bytes".into(),
            "parse".into(),
            "bogus".into(),
            "--json".into(),
        ])
        .unwrap_err();
        assert!(err.contains("bogus"));
    }

    #[test]
    fn json_escape_handles_quotes_and_backslashes() {
        assert_eq!(json_escape(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(json_escape("tab\there"), "tab\\there");
    }
}
