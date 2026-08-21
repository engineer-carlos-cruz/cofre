mod app;
mod errors;
mod terminal;
mod ui;

use app::AppState;
use errors::CofreError;
use std::io::IsTerminal;
use std::process::ExitCode;
use terminal::{init, teardown};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cofre: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), CofreError> {
    if !std::io::stdout().is_terminal() {
        return Err(CofreError::NotTty);
    }
    let state = AppState::default();
    let mut guard = init()?;
    let _frame = ui::draw(&state);
    teardown(&mut guard)
}
