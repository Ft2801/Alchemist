//! Utility functions for string manipulation and naming

/// Convert a string to PascalCase
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert a string to snake_case
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut last_was_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 && !last_was_upper {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            last_was_upper = true;
        } else if c == '-' || c == ' ' {
            result.push('_');
            last_was_upper = false;
        } else {
            result.push(c);
            last_was_upper = false;
        }
    }

    result
}

/// Convert a string to a safe identifier (handling keywords and invalid chars)
pub fn to_safe_identifier(name: &str) -> String {
    let mut safe = name.replace('-', "_");

    // Add underscore if starts with number
    if safe.chars().next().is_some_and(|c| c.is_numeric()) {
        safe.insert(0, '_');
    }

    // Escape rust keywords (basic list)
    match safe.as_str() {
        "type" | "struct" | "enum" | "fn" | "impl" | "trait" | "match" | "if" | "else"
        | "while" | "for" | "loop" | "return" | "break" | "continue" | "let" | "mut" | "const"
        | "static" | "pub" | "mod" | "use" | "crate" | "super" | "self" | "Self" => {
            format!("r#{}", safe)
        }
        _ => safe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user_name"), "UserName");
        assert_eq!(to_pascal_case("first-name"), "FirstName");
        assert_eq!(to_pascal_case("hello world"), "HelloWorld");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("UserName"), "user_name");
        assert_eq!(to_snake_case("first-name"), "first_name");
        assert_eq!(to_snake_case("HTMLParser"), "htmlparser"); // basic implementation
    }
}
