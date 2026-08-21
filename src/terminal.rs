use crate::errors::CofreError;

pub struct TerminalGuard;

pub fn init() -> Result<TerminalGuard, CofreError> {
    Ok(TerminalGuard)
}

pub fn teardown(_guard: &mut TerminalGuard) -> Result<(), CofreError> {
    Ok(())
}
