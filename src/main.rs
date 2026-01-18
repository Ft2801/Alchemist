//! Alchemist - Transform JSON/YAML/TOML into type-safe code

mod ast;
mod cli;
mod error;
mod formats;
mod generators;
mod parser;
mod reporter;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use formats::OutputFormat;
use generators::CodeGenerator;
use owo_colors::set_override;
use reporter::{ConversionStats, Reporter};
use std::fs;
use std::time::Instant;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle shell completions request
    if let Some(shell) = cli.completions {
        Cli::print_completions(shell);
        return Ok(());
    }

    // Handle no-color mode for CI/CD
    if cli.no_color {
        set_override(false);
    }

    // Start timing
    let start = Instant::now();

    // Read input from file or stdin
    let input_content = cli.read_input()?;
    let input_size = input_content.len();

    // Create generator options
    let options = cli.generator_options();

    // Auto-detect input format from extension, or use specified
    let input_format = cli.detect_input_format();

    // Parse input to AST based on input format
    let schema = match input_format {
        formats::InputFormat::Json => parser::parse_json(&input_content, &options),
        formats::InputFormat::Yaml => parser::parse_yaml(&input_content, &options),
        formats::InputFormat::Toml => parser::parse_toml(&input_content, &options),
    };

    let schema = match schema {
        Ok(s) => s,
        Err(e) => {
            Reporter::print_error(&e.to_string());
            return Err(e.into());
        }
    };

    // Select generator based on output format
    let generator: Box<dyn CodeGenerator> = match cli.output_format {
        OutputFormat::Rust => Box::new(generators::rust::RustGenerator::new(
            cli.generator_options(),
        )),
        OutputFormat::Typescript => Box::new(generators::typescript::TypeScriptGenerator::new(
            cli.generator_options(),
        )),
        OutputFormat::Zod => Box::new(generators::zod::ZodGenerator::new(cli.generator_options())),
        OutputFormat::Python => Box::new(generators::python::PythonGenerator::new(
            cli.generator_options(),
        )),
    };

    let output = generator.generate(&schema)?;

    let output_size = output.len();
    let duration = start.elapsed();

    // Calculate statistics
    let stats = ConversionStats::from_schema(&schema, duration, input_size, output_size);

    // Write output to file if specified
    if let Some(ref output_path) = cli.output {
        fs::write(output_path, &output)?;
    }

    // Print report and output
    if !cli.quiet {
        Reporter::print_stats(
            &stats,
            &format!("{} (.{})", generator.name(), generator.file_extension()),
        );
        Reporter::print_types_summary(&schema);
        Reporter::print_success(cli.output.as_ref().map(|p| p.to_str().unwrap_or("output")));

        // Print generated code to stdout only if no output file specified
        if cli.output.is_none() {
            println!("{}", "─".repeat(60));
            println!();
            println!("{}", output);
        }
    } else if cli.output.is_none() {
        // Quiet mode but no output file - just print the code
        print!("{}", output);
    }

    Ok(())
}
