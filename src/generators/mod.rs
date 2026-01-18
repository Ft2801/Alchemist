//! Code generators module

pub mod python;
pub mod rust;
pub mod typescript;
pub mod zod;

use crate::ast::Schema;
use crate::error::Result;

/// Options for code generation
#[derive(Debug, Clone)]
pub struct GeneratorOptions {
    /// Root type name
    pub root_name: String,
    /// Whether to generate optional fields
    pub optional_fields: bool,
    /// Whether to use readonly modifier (TypeScript)
    pub readonly: bool,
    /// Derive macros to add (Rust)
    pub derive_macros: Vec<String>,
    /// Whether to use pub modifier for fields (Rust)
    pub public_fields: bool,
}

impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            root_name: "Root".to_string(),
            optional_fields: false,
            readonly: false,
            derive_macros: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "Serialize".to_string(),
                "Deserialize".to_string(),
            ],
            public_fields: true,
        }
    }
}

/// Trait for code generators
///
/// This trait defines the interface for generating code from an intermediate AST.
/// Each target language (Rust, TypeScript, Zod) implements this trait to produce
/// language-specific output.
///
/// # Example
///
/// ```ignore
/// use alchemist::generators::{CodeGenerator, GeneratorOptions};
/// use alchemist::generators::typescript::TypeScriptGenerator;
/// use alchemist::ast::Schema;
///
/// let schema = Schema::new("User");
/// let generator = TypeScriptGenerator::new(GeneratorOptions::default());
/// let code = generator.generate(&schema)?;
/// ```
pub trait CodeGenerator {
    /// Generate code from the given schema AST
    ///
    /// # Arguments
    ///
    /// * `schema` - The intermediate AST representation of the data structure
    ///
    /// # Returns
    ///
    /// Returns a `Result<String>` containing the generated code or an error
    /// if code generation fails.
    fn generate(&self, schema: &Schema) -> Result<String>;

    /// Get the file extension for the generated code
    ///
    /// # Returns
    ///
    /// Returns the appropriate file extension (e.g., "rs", "ts", "ts" for Zod)
    fn file_extension(&self) -> &'static str;

    /// Get the name of the generator
    ///
    /// # Returns
    ///
    /// Returns a human-readable name for the generator
    fn name(&self) -> &'static str;
}
