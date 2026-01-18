//! Input and Output format enums

use clap::ValueEnum;
use std::fmt;

/// Supported input formats for parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InputFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
}

impl fmt::Display for InputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputFormat::Json => write!(f, "json"),
            InputFormat::Yaml => write!(f, "yaml"),
            InputFormat::Toml => write!(f, "toml"),
        }
    }
}

/// Supported output formats for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Rust structs with serde derive macros
    Rust,
    /// TypeScript interfaces
    Typescript,
    /// Zod schema validation
    Zod,
    /// Python Pydantic models
    Python,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Rust => write!(f, "rust"),
            OutputFormat::Typescript => write!(f, "typescript"),
            OutputFormat::Zod => write!(f, "zod"),
            OutputFormat::Python => write!(f, "python"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_format_display() {
        assert_eq!(InputFormat::Json.to_string(), "json");
        assert_eq!(InputFormat::Yaml.to_string(), "yaml");
        assert_eq!(InputFormat::Toml.to_string(), "toml");
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Rust.to_string(), "rust");
        assert_eq!(OutputFormat::Typescript.to_string(), "typescript");
        assert_eq!(OutputFormat::Zod.to_string(), "zod");
        assert_eq!(OutputFormat::Python.to_string(), "python");
    }
}
