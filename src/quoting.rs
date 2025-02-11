use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum QuotingError {
    #[error("Dangling quote encountered")]
    DanglingQuote,
}

const ESCAPE_CHARACTER: char = '\\';
const ESCAPABLE_DOUBLE_QUOTED_CHARACTERS: [char; 4] = [DOUBLE_QUOTE, '\\', '$', '\n'];
const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';
const NEWLINE: char = '\n';

/// Split the provided string, taking into account single-quoting, double-quoting, and escaping
/// rules.
pub(crate) fn split_quoted_string(input: &str) -> Result<Vec<String>, QuotingError> {
    // Split arguments separated by spaces, apart if they are single-quoted.
    let mut split_args = Vec::new();
    let mut current_arg = String::new();

    let mut is_within_quotes = false;
    let mut is_within_double_quotes = false;
    let mut is_escaping = false;

    for char in input.chars() {
        if is_arg_boundary(char, &current_arg, is_within_quotes, is_escaping) {
            // Split the argument at this character, skipping the character itself.
            split_args.push(current_arg);
            current_arg = String::new();
        } else if is_escaping {
            if is_within_double_quotes && !ESCAPABLE_DOUBLE_QUOTED_CHARACTERS.contains(&char) {
                // Push the escape character.
                current_arg.push(ESCAPE_CHARACTER);
            }

            // Push the current character if not a newline.
            if char != NEWLINE {
                current_arg.push(char);
            }

            // Disable escape mode.
            is_escaping = false;
        } else if is_double_quoting_toggle(char, is_within_double_quotes, is_within_quotes) {
            // Toggle double-quoted and quoted mode mode.
            is_within_double_quotes = !is_within_double_quotes;
            is_within_quotes = !is_within_quotes;
        } else if is_single_quoting_toggle(char, is_within_double_quotes) {
            // Toggle quoted mode.
            is_within_quotes = !is_within_quotes;
        } else if is_escaping_toggle(char, is_within_double_quotes, is_within_quotes) {
            // Enable escape mode.
            is_escaping = true;
        } else if should_capture_char(char, is_within_quotes) {
            // Capture characters.
            current_arg.push(char);
        }
    }

    if !current_arg.is_empty() {
        split_args.push(current_arg);
    }

    if is_within_quotes {
        return Err(QuotingError::DanglingQuote);
    }

    Ok(split_args)
}

fn should_capture_char(current_char: char, is_within_quotes: bool) -> bool {
    // Skip whitespaces outside quoted strings.
    is_within_quotes || !current_char.is_whitespace()
}

fn is_escaping_toggle(
    current_char: char,
    is_within_double_quotes: bool,
    is_within_quotes: bool,
) -> bool {
    // Only interpret backslashes if they are not within a single-quoted string.
    (!is_within_quotes || is_within_double_quotes) && current_char == ESCAPE_CHARACTER
}

fn is_single_quoting_toggle(current_char: char, is_within_double_quotes: bool) -> bool {
    // Only interpret single-quotes if they are not within a double-quoted string.
    !is_within_double_quotes && current_char == SINGLE_QUOTE
}

fn is_double_quoting_toggle(
    current_char: char,
    is_within_double_quotes: bool,
    is_within_quotes: bool,
) -> bool {
    // Only interpret double-quotes if they are not within a single-quoted string.
    (!is_within_quotes || is_within_double_quotes) && current_char == DOUBLE_QUOTE
}

fn is_arg_boundary(
    current_char: char,
    current_arg: &str,
    is_within_quotes: bool,
    is_escaping: bool,
) -> bool {
    // Break at whitespaces when not within quotes, and the whitespace is not being escaped.
    !is_escaping && !is_within_quotes && current_char.is_whitespace() && !current_arg.is_empty()
}

#[cfg(test)]
mod tests {
    use crate::quoting::{split_quoted_string, QuotingError};

    #[test]
    fn it_splits_command_from_args() {
        // Split at spaces.
        assert_eq!(
            ["hello", "world"].map(str::to_owned).to_vec(),
            split_quoted_string("hello world").unwrap()
        );
        assert_eq!(
            ["hello", "world"].map(str::to_owned).to_vec(),
            split_quoted_string("hello       world").unwrap()
        );
    }

    #[test]
    fn it_splits_single_quoted_args() {
        // Don't split at spaces within single-quoted strings.
        assert_eq!(
            ["hello", "to the world", "from ", "me"]
                .map(str::to_owned)
                .to_vec(),
            split_quoted_string("hello 'to the world'     'from ' me").unwrap()
        );

        // Don't split args at single quotes if not surrounded by spaces.
        assert_eq!(
            ["hello", "world"].map(str::to_owned).to_vec(),
            split_quoted_string("hello w'orl'd").unwrap()
        );
        assert_eq!(
            ["hello", "world"].map(str::to_owned).to_vec(),
            split_quoted_string("hello 'worl'd").unwrap()
        );
        assert_eq!(
            ["hello", "world oh"].map(str::to_owned).to_vec(),
            split_quoted_string("hello wo'rld 'oh").unwrap()
        );

        // Error on dangling single-quoted string.
        assert!(matches!(
            split_quoted_string("hello 'world"),
            Err(QuotingError::DanglingQuote)
        ));
    }

    #[test]
    fn it_splits_double_quoted_args_similarly_to_single_quotes() {
        // Don't split at spaces within double-quoted strings.
        assert_eq!(
            ["hello", "to the world", "from ", "me"]
                .map(str::to_owned)
                .to_vec(),
            split_quoted_string(r#"hello "to the world"     "from " me"#).unwrap()
        );

        // Don't split args at double quotes if not surrounded by spaces.
        assert_eq!(
            ["hello", "world"].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello w"orl"d"#).unwrap()
        );
        assert_eq!(
            ["hello", "world"].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello "worl"d"#).unwrap()
        );
        assert_eq!(
            ["hello", "world oh"].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello wo"rld "oh"#).unwrap()
        );
    }

    #[test]
    fn it_preserves_the_literal_value_of_characters_within_single_quotes() {
        // Preserve double-quotes.
        assert_eq!(
            ["hello", r#"to "the" world"#].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello 'to "the" world'"#).unwrap()
        );

        // Preserve backslashes.
        assert_eq!(
            [r#"hello\\\\world"#].map(str::to_owned).to_vec(),
            split_quoted_string(r#"'hello\\\\world'"#).unwrap()
        );
        assert_eq!(
            ["hello", r#"to \"the\" world"#].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello 'to \"the\" world'"#).unwrap()
        );
    }

    #[test]
    fn it_preserves_the_literal_value_of_characters_within_double_quotes() {
        // Preserve single-quotes.
        assert_eq!(
            ["hello", "to 'the' world"].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello "to 'the' world""#).unwrap()
        );
        assert_eq!(
            ["hello", "wo'r'ld"].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello w"o'r'l"d"#).unwrap()
        );
    }

    #[test]
    fn it_handles_escaping_within_double_quotes() {
        // Escape double-quotes.
        assert_eq!(
            ["hello", r#"to "the" world"#].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello "to \"the\" world""#).unwrap()
        );

        // Escape backslash.
        assert_eq!(
            [r#"he\\o"#].map(str::to_owned).to_vec(),
            split_quoted_string(r#""he\\\\o""#).unwrap()
        );

        // Escape dollar.
        assert_eq!(
            ["hello", "$HOME"].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello "\$HOME""#).unwrap()
        );

        // Escape newline, treating it as a continuation.
        assert_eq!(
            ["hello", "to the world"].map(str::to_owned).to_vec(),
            split_quoted_string(
                r#"hello "to the \
world""#
            )
            .unwrap()
        );

        // Does NOT escape backslash if not followed by one of \, ", $.
        assert_eq!(
            ["hello", r#"wor\d"#].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello "wor\d""#).unwrap()
        );
    }

    #[test]
    fn it_handles_escaping_outside_double_quotes() {
        // Escape whitespace.
        assert_eq!(
            ["hello   world"].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello\ \ \ world"#).unwrap()
        );

        // Escape single-quoting.
        assert_eq!(
            ["hello", "'world'"].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello \'world\'"#).unwrap()
        );

        // Escape double-quoting.
        assert_eq!(
            ["hello", r#""world""#].map(str::to_owned).to_vec(),
            split_quoted_string(r#"hello \"world\""#).unwrap()
        );

        // Escape newline, treating it as a continuation.
        assert_eq!(
            ["hello", "to", "the", "world"].map(str::to_owned).to_vec(),
            split_quoted_string(
                r#"hello to \
the world"#
            )
            .unwrap()
        );

        // Escape backslash.
        assert_eq!(
            [r#"he\\o"#, r#"wor\d"#].map(str::to_owned).to_vec(),
            split_quoted_string(r#"he\\\\o wor\\d"#).unwrap()
        );

        // Does NOT print the backslash when not escaping itself.
        assert_eq!(
            ["heo", "word"].map(str::to_owned).to_vec(),
            split_quoted_string(r#"he\o wor\d"#).unwrap()
        );
    }
}
