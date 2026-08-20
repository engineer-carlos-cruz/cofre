use crate::app::Screen;

pub fn header_for(screen: &Screen) -> &'static str {
    match screen {
        Screen::Unlock => "[unlock]",
        Screen::List => "[list]",
        Screen::Detail => "[detail]",
        Screen::Form => "[form]",
        Screen::Generator => "[generator]",
        Screen::Settings => "[settings]",
    }
}

pub fn placeholder_body(screen: &Screen) -> &'static str {
    match screen {
        Screen::Unlock => "Pantalla de desbloqueo (placeholder)",
        Screen::List => "Listado de entradas (placeholder)",
        Screen::Detail => "Detalle de entrada (placeholder)",
        Screen::Form => "Formulario (placeholder)",
        Screen::Generator => "Generador (placeholder)",
        Screen::Settings => "Ajustes (placeholder)",
    }
}