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
    match args {
        [cmd, sub, value] if cmd == "bytes" && sub == "parse" => {
            let size = ByteSize::from_str(value).map_err(|e| e.to_string())?;
            Ok(format!("{} bytes", size.as_bytes()))
        }
        [cmd, sub, value] if cmd == "bytes" && sub == "fmt" => {
            let n: u64 = value
                .parse()
                .map_err(|_| format!("invalid byte count: {value:?}"))?;
            Ok(ByteSize::from_bytes(n).to_string())
        }
        [cmd, sub, value] if cmd == "duration" && sub == "parse" => {
            let d = parse_duration(value).map_err(|e| e.to_string())?;
            Ok(format!("{} ms", d.as_millis()))
        }
        [cmd, sub, value] if cmd == "duration" && sub == "fmt" => {
            let secs: f64 = value
                .parse()
                .map_err(|_| format!("invalid seconds: {value:?}"))?;
            if secs < 0.0 || !secs.is_finite() {
                return Err("duration cannot be negative".to_string());
            }
            Ok(format_duration(Duration::from_secs_f64(secs)))
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage:\n  \
     sizespan bytes parse <text>      e.g. sizespan bytes parse 1.5MiB\n  \
     sizespan bytes fmt <n>           e.g. sizespan bytes fmt 1572864\n  \
     sizespan duration parse <text>   e.g. sizespan duration parse 1h30m\n  \
     sizespan duration fmt <seconds>  e.g. sizespan duration fmt 5400"
        .to_string()
}
