pub(crate) fn parse_args(args_string: &str) -> Result<Vec<String>, String> {
    // Split arguments separated by spaces, apart if they are single-quoted.
    let mut split_args = Vec::new();
    let mut current = "".to_owned();
    let mut is_within_quotes = false;
    let mut is_within_double_quotes = false;
    for char in args_string.chars() {
        if !is_within_quotes && char.is_whitespace() && !current.is_empty() {
            // Break at whitespaces when not within quotes.
            split_args.push(current);
            current = "".to_owned();
        } else if (!is_within_quotes || is_within_double_quotes) && char == '"' {
            // Toggle double-quoted and quoted mode mode.
            is_within_double_quotes = !is_within_double_quotes;
            is_within_quotes = !is_within_quotes;
        } else if !is_within_double_quotes && char == '\'' {
            // Toggle quoted mode.
            is_within_quotes = !is_within_quotes;
        } else if is_within_quotes || !char.is_whitespace() {
            // Capture characters.
            // Skip whitespaces outside of quotes.
            current.push(char);
        }
    }

    if is_within_quotes {
        return Err("Invalid single-quoting".to_owned());
    } else if !current.is_empty() {
        split_args.push(current);
    }

    Ok(split_args)
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

        // Skip single quotes if not separated by spaces.
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

        // Skip double quotes if not separated by spaces.
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
    }

    #[test]
    fn it_handles_escaping_within_double_quotes() {
        // Escape double-quotes.
        assert_eq!(
            Ok(["hello", "to \"the\" world"].map(str::to_owned).to_vec()),
            parse_args(r#"hello "to \"the\" world""#)
        );
    }


}
