use crate::builtin::BuiltInCommand;
use crate::path::{find_partial_executable_matches_in_path, PathError};
use std::collections::HashSet;
use strum::VariantNames;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum AutocompleteError {
    #[error("Failed to find executables in the PATH: {0}")]
    Path(#[from] PathError),
}

pub(crate) trait Autocomplete {
    fn completions(&self, input: &str) -> Result<HashSet<String>, AutocompleteError>;
}

pub(crate) struct CompositeAutocomplete {
    autocompletes: Vec<Box<dyn Autocomplete>>,
}

impl CompositeAutocomplete {
    pub(crate) fn new() -> Self {
        Self {
            autocompletes: vec![
                Box::new(BuiltInAutocompletion {}),
                Box::new(PathAutocompletion {}),
            ],
        }
    }
}

impl Autocomplete for CompositeAutocomplete {
    fn completions(&self, input: &str) -> Result<HashSet<String>, AutocompleteError> {
        // Collect into a HashSet to deduplicate entries.
        let completions: HashSet<_> = self
            .autocompletes
            .iter()
            // Collect completions from every autocomplete.
            .map(|autocomplete| autocomplete.completions(input))
            // Bubble up errors.
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            // Return completions as a flat list.
            .flatten()
            .collect();

        Ok(completions)
    }
}

struct BuiltInAutocompletion {}

impl Autocomplete for BuiltInAutocompletion {
    fn completions(&self, input: &str) -> Result<HashSet<String>, AutocompleteError> {
        let builtins = BuiltInCommand::VARIANTS
            .iter()
            .filter(|cmd| cmd.starts_with(input))
            .map(ToString::to_string)
            .collect();

        Ok(builtins)
    }
}

struct PathAutocompletion {}

impl Autocomplete for PathAutocompletion {
    fn completions(&self, input: &str) -> Result<HashSet<String>, AutocompleteError> {
        let path_executables = find_partial_executable_matches_in_path(input)?;

        Ok(path_executables)
    }
}

#[cfg(test)]
mod tests {
    use crate::autocomplete::{Autocomplete, BuiltInAutocompletion};
    use std::collections::HashSet;

    #[test]
    fn it_autocompletes_builtin() {
        let builtin_autocompletion = BuiltInAutocompletion {};

        // With exactly one match.
        assert_eq!(
            HashSet::from(["echo".to_owned()]),
            builtin_autocompletion.completions("ech").unwrap()
        );
        assert_eq!(
            HashSet::from(["echo".to_owned()]),
            builtin_autocompletion.completions("echo").unwrap()
        );
        assert_eq!(
            HashSet::from(["exit".to_owned()]),
            builtin_autocompletion.completions("ex").unwrap()
        );

        // With no match at all.
        assert_eq!(
            HashSet::<String>::new(),
            builtin_autocompletion
                .completions("non_existent_function")
                .unwrap()
        );

        // Abort when multiple matches.
        assert_eq!(
            HashSet::from(["echo".to_owned(), "exit".to_owned()]),
            builtin_autocompletion.completions("e").unwrap()
        );
    }
}
