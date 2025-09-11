use anyhow::Result;

/// Useful for using the `?` result syntax in functions which can only end with an error but won't return
/// normally. I.e. they return `Result<Never, SomeError>`.
/// We don't use `Infalliable` here, because the wording doesn't fit well with the positive case.
pub enum Never {}

pub fn get_err(result: Result<Never>) -> anyhow::Error {
    match result {
        Err(e) => e,
        Ok(_) => unreachable!("never value observerd.")
    }
}
