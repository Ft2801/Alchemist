//! Error types for Alchemist

use thiserror::Error;

/// Main error type for Alchemist operations
#[derive(Error, Debug)]
pub enum AlchemistError {
    /// Error parsing JSON input
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    /// Error parsing YAML input
    #[error("Failed to parse YAML: {0}")]
    YamlParseError(#[from] serde_yaml::Error),

    /// Error during code generation
    #[error("Code generation failed: {0}")]
    GenerationError(String),

    /// Invalid input structure
    #[error("Invalid input structure: {0}")]
    InvalidStructure(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type alias for Alchemist operations
pub type Result<T> = std::result::Result<T, AlchemistError>;
