use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum AutocompleteError {
    #[error("Failed to read the input: {0:?}")]
    ReadInputFailed(std::io::Error),
}

pub(crate) fn capture_input() -> Result<String, AutocompleteError> {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(AutocompleteError::ReadInputFailed)?;

    Ok(input.trim().to_owned())
}
