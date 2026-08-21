#[allow(dead_code)]
pub enum Screen {
    Unlock,
    List,
    Detail,
    Form,
    Generator,
    Settings,
}

pub struct AppState {
    pub screen: Screen,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Unlock,
        }
    }
}
