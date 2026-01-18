//! CLI argument definitions using clap

use crate::formats::{InputFormat, OutputFormat};
use crate::generators::GeneratorOptions;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::io::{self, Read};
use std::path::PathBuf;

/// Alchemist - Transform JSON/YAML/TOML into type-safe code
///
/// A blazingly fast CLI tool for generating Rust structs, TypeScript interfaces,
/// Zod schemas, and Python Pydantic models from JSON, YAML, or TOML input.
#[derive(Parser, Debug)]
#[command(name = "alchemist")]
#[command(author = "Fabio Tempera")]
#[command(version)]
#[command(about = "Transform JSON/YAML/TOML into Rust, TypeScript, Zod, or Python code")]
#[command(
    long_about = "Alchemist is a blazingly fast CLI tool that converts JSON, YAML, or TOML data into type-safe code structures.\n\nSupported outputs:\n  • Rust structs with serde derive macros\n  • TypeScript interfaces\n  • Zod validation schemas\n  • Python Pydantic models\n\nExamples:\n  alchemist -i data.json\n  alchemist -i config.yaml -f yaml -t rust\n  cat data.json | alchemist -t python\n  alchemist --completions bash > ~/.local/share/bash-completion/completions/alchemist"
)]
pub struct Cli {
    /// Input file path. Use '-' or omit to read from stdin
    #[arg(short, long)]
    pub input: Option<PathBuf>,

    /// Output file path (prints to stdout if not provided)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Input format (auto-detected from extension if not specified)
    #[arg(short = 'f', long, default_value = "json")]
    pub input_format: InputFormat,

    /// Output format
    #[arg(short = 't', long, default_value = "typescript")]
    pub output_format: OutputFormat,

    /// Root type name for the generated code
    #[arg(short = 'n', long, default_value = "Root")]
    pub root_name: String,

    /// Generate optional fields (for TypeScript/Python)
    #[arg(long)]
    pub optional_fields: bool,

    /// Use readonly modifier (for TypeScript)
    #[arg(long)]
    pub readonly: bool,

    /// Add derive macros (for Rust, comma-separated)
    #[arg(long, default_value = "Debug,Clone,Serialize,Deserialize")]
    pub derive: String,

    /// Use pub modifier for fields (for Rust)
    #[arg(long, default_value = "true")]
    pub public_fields: bool,

    /// Quiet mode - suppress visual report, only output generated code
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Disable colored output (useful for CI/CD pipelines)
    #[arg(long)]
    pub no_color: bool,

    /// Generate shell completions for the specified shell
    #[arg(long, value_name = "SHELL")]
    pub completions: Option<Shell>,
}

impl Cli {
    /// Convert CLI arguments to GeneratorOptions
    pub fn generator_options(&self) -> GeneratorOptions {
        GeneratorOptions {
            root_name: self.root_name.clone(),
            optional_fields: self.optional_fields,
            readonly: self.readonly,
            derive_macros: self
                .derive
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            public_fields: self.public_fields,
        }
    }

    /// Read input content from file or stdin
    pub fn read_input(&self) -> io::Result<String> {
        match &self.input {
            Some(path) if path.to_string_lossy() != "-" => std::fs::read_to_string(path),
            _ => {
                // Read from stdin
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                Ok(buffer)
            }
        }
    }

    /// Generate shell completions and print to stdout
    pub fn print_completions(shell: Shell) {
        let mut cmd = Self::command();
        generate(shell, &mut cmd, "alchemist", &mut io::stdout());
    }

    /// Auto-detect input format from file extension
    pub fn detect_input_format(&self) -> InputFormat {
        if let Some(path) = &self.input {
            if let Some(ext) = path.extension() {
                return match ext.to_string_lossy().to_lowercase().as_str() {
                    "yaml" | "yml" => InputFormat::Yaml,
                    "toml" => InputFormat::Toml,
                    _ => self.input_format,
                };
            }
        }
        self.input_format
    }
}
