use std::error::Error;
use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum CofreError {
    NotTty,
    TermNotSet,
    RawMode,
    AlternateScreen,
    Teardown,
}

impl fmt::Display for CofreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mensaje = match self {
            CofreError::NotTty => "no es una terminal interactiva",
            CofreError::TermNotSet => "la variable de entorno TERM no está definida",
            CofreError::RawMode => "no se pudo activar el modo raw de la terminal",
            CofreError::AlternateScreen => "no se pudo entrar en la pantalla alternativa",
            CofreError::Teardown => "error al restaurar la terminal",
        };
        write!(f, "{mensaje}")
    }
}

impl Error for CofreError {}

#[cfg(test)]
mod test_errors {
    use super::*;

    #[test]
    fn variantes_producen_mensaje_no_vacio() {
        let variantes = [
            CofreError::NotTty,
            CofreError::TermNotSet,
            CofreError::RawMode,
            CofreError::AlternateScreen,
            CofreError::Teardown,
        ];
        for variante in variantes {
            assert!(!variante.to_string().is_empty());
        }
    }
}
