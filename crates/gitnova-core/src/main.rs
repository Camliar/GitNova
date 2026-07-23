use std::io::{self, BufReader, BufWriter};
use std::process::ExitCode;

fn main() -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    match gitnova_core::run(&mut reader, &mut writer) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::from(1),
        Err(error) => {
            eprintln!("gitnova-core transport error: {error}");
            ExitCode::from(1)
        }
    }
}
