pub mod screens;

use crate::app::AppState;

pub fn draw(state: &AppState) -> String {
    let header = screens::header_for(&state.screen);
    let body = screens::placeholder_body(&state.screen);
    format!("{header}\n{body}")
}