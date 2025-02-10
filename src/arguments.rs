const ESCAPE_CHARACTER: char = '\\';
const ESCAPABLE_DOUBLE_QUOTED_CHARACTERS: [char; 4] = [DOUBLE_QUOTE, '\\', '$', '\n'];
const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';
const NEWLINE: char = '\n';

pub(crate) fn parse_args(args_string: &str) -> Result<Vec<String>, String> {
    // Split arguments separated by spaces, apart if they are single-quoted.
    let mut split_args = Vec::new();
    let mut current_arg = String::new();

    let mut is_within_quotes = false;
    let mut is_within_double_quotes = false;
    let mut is_escaping = false;

    for char in args_string.chars() {
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
        } else if is_escaping_toggle(char) {
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
        return Err("Invalid single-quoting".to_owned());
    }

    Ok(split_args)
}

fn should_capture_char(current_char: char, is_within_quotes: bool) -> bool {
    // Skip whitespaces outside quoted strings.
    is_within_quotes || !current_char.is_whitespace()
}

fn is_escaping_toggle(current_char: char) -> bool {
    current_char == ESCAPE_CHARACTER
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
    use crate::arguments::parse_args;

    #[test]
    fn it_splits_command_from_args() {
        // Split at spaces.
        assert_eq!(
            Ok(["hello", "world"].map(str::to_owned).to_vec()),
            parse_args("hello world")
        );
        assert_eq!(
            Ok(["hello", "world"].map(str::to_owned).to_vec()),
            parse_args("hello       world")
        );
    }

    #[test]
    fn it_splits_single_quoted_args() {
        // Don't split at spaces within single-quoted strings.
        assert_eq!(
            Ok(["hello", "to the world", "from ", "me"]
                .map(str::to_owned)
                .to_vec()),
            parse_args("hello 'to the world'     'from ' me")
        );

        // Don't split args at single quotes if not surrounded by spaces.
        assert_eq!(
            Ok(["hello", "world"].map(str::to_owned).to_vec()),
            parse_args("hello w'orl'd")
        );
        assert_eq!(
            Ok(["hello", "world"].map(str::to_owned).to_vec()),
            parse_args("hello 'worl'd")
        );
        assert_eq!(
            Ok(["hello", "world oh"].map(str::to_owned).to_vec()),
            parse_args("hello wo'rld 'oh")
        );

        // Error on dangling single-quoted string.
        assert_eq!(
            Err("Invalid single-quoting".to_owned()),
            parse_args("hello 'world")
        );
    }

    #[test]
    fn it_splits_double_quoted_args_similarly_to_single_quotes() {
        // Don't split at spaces within double-quoted strings.
        assert_eq!(
            Ok(["hello", "to the world", "from ", "me"]
                .map(str::to_owned)
                .to_vec()),
            parse_args(r#"hello "to the world"     "from " me"#)
        );

        // Don't split args at double quotes if not surrounded by spaces.
        assert_eq!(
            Ok(["hello", "world"].map(str::to_owned).to_vec()),
            parse_args(r#"hello w"orl"d"#)
        );
        assert_eq!(
            Ok(["hello", "world"].map(str::to_owned).to_vec()),
            parse_args(r#"hello "worl"d"#)
        );
        assert_eq!(
            Ok(["hello", "world oh"].map(str::to_owned).to_vec()),
            parse_args(r#"hello wo"rld "oh"#)
        );
    }

    #[test]
    fn it_preserves_the_literal_value_of_characters_within_double_quotes() {
        // Preserve single-quotes.
        assert_eq!(
            Ok(["hello", "to 'the' world"].map(str::to_owned).to_vec()),
            parse_args(r#"hello "to 'the' world""#)
        );
        assert_eq!(
            Ok(["hello", "wo'r'ld"].map(str::to_owned).to_vec()),
            parse_args(r#"hello w"o'r'l"d"#)
        );
    }

    #[test]
    fn it_handles_escaping_within_double_quotes() {
        // Escape double-quotes.
        assert_eq!(
            Ok(["hello", r#"to "the" world"#].map(str::to_owned).to_vec()),
            parse_args(r#"hello "to \"the\" world""#)
        );

        // Escape backslash.
        assert_eq!(
            Ok([r#"he\\o"#].map(str::to_owned).to_vec()),
            parse_args(r#""he\\\\o""#)
        );

        // Escape dollar.
        assert_eq!(
            Ok(["hello", "$HOME"].map(str::to_owned).to_vec()),
            parse_args(r#"hello "\$HOME""#)
        );

        // Escape newline, treating it as a continuation.
        assert_eq!(
            Ok(["hello", "to the world"].map(str::to_owned).to_vec()),
            parse_args(
                r#"hello "to the \
world""#
            )
        );

        // Does NOT escape backslash if not followed by one of \, ", $.
        assert_eq!(
            Ok(["hello", r#"wor\d"#].map(str::to_owned).to_vec()),
            parse_args(r#"hello "wor\d""#)
        );
    }

    #[test]
    fn it_handles_escaping_outside_double_quotes() {
        // Escape whitespace.
        assert_eq!(
            Ok(["hello   world"].map(str::to_owned).to_vec()),
            parse_args(r#"hello\ \ \ world"#)
        );

        // Escape single-quoting.
        assert_eq!(
            Ok(["hello", "'world'"].map(str::to_owned).to_vec()),
            parse_args(r#"hello \'world\'"#)
        );

        // Escape double-quoting.
        assert_eq!(
            Ok(["hello", r#""world""#].map(str::to_owned).to_vec()),
            parse_args(r#"hello \"world\""#)
        );

        // Escape newline, treating it as a continuation.
        assert_eq!(
            Ok(["hello", "to", "the", "world"].map(str::to_owned).to_vec()),
            parse_args(
                r#"hello to \
the world"#
            )
        );

        // Escape backslash.
        assert_eq!(
            Ok([r#"he\\o"#, r#"wor\d"#].map(str::to_owned).to_vec()),
            parse_args(r#"he\\\\o wor\\d"#)
        );

        // Does NOT print the backslash when not escaping itself.
        assert_eq!(
            Ok(["heo", "word"].map(str::to_owned).to_vec()),
            parse_args(r#"he\o wor\d"#)
        );
    }
}
