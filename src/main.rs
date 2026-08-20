mod app;
mod errors;
mod terminal;
mod ui;

use app::AppState;
use errors::CofreError;
use terminal::{init, teardown};

fn main() -> Result<(), CofreError> {
    run()
}

fn run() -> Result<(), CofreError> {
    let state = AppState::default();
    let mut guard = init()?;
    let _render = ui::draw(&state);
    teardown(&mut guard)
}