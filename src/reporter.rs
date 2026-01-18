//! Visual reporting module for beautiful terminal output
//!
//! Uses owo-colors to create colorful, informative reports about
//! the conversion process.

use crate::ast::{FieldType, Schema};
use owo_colors::OwoColorize;
use std::time::Duration;

/// Statistics collected during the conversion process
#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    /// Total time taken for the conversion
    pub duration: Duration,
    /// Number of types generated
    pub types_count: usize,
    /// Total number of fields across all types
    pub fields_count: usize,
    /// Number of optional fields
    pub optional_fields_count: usize,
    /// Number of nested types (non-root types)
    pub nested_types_count: usize,
    /// Maximum nesting depth
    pub max_depth: usize,
    /// Number of array fields
    pub array_fields_count: usize,
    /// Input file size in bytes
    pub input_size: usize,
    /// Output size in bytes
    pub output_size: usize,
}

impl ConversionStats {
    /// Calculate statistics from a schema
    pub fn from_schema(
        schema: &Schema,
        duration: Duration,
        input_size: usize,
        output_size: usize,
    ) -> Self {
        let types_count = schema.types.len();
        let nested_types_count = types_count.saturating_sub(1);

        let mut fields_count = 0;
        let mut optional_fields_count = 0;
        let mut array_fields_count = 0;
        let mut max_depth = 0;

        for type_def in &schema.types {
            fields_count += type_def.fields.len();

            for field in &type_def.fields {
                if field.optional {
                    optional_fields_count += 1;
                }

                if matches!(field.field_type, FieldType::Array(_)) {
                    array_fields_count += 1;
                }

                // Calculate depth for this field
                let depth = calculate_type_depth(&field.field_type);
                max_depth = max_depth.max(depth);
            }
        }

        Self {
            duration,
            types_count,
            fields_count,
            optional_fields_count,
            nested_types_count,
            max_depth,
            array_fields_count,
            input_size,
            output_size,
        }
    }

    /// Calculate complexity score (1-10)
    pub fn complexity_score(&self) -> u8 {
        let mut score: u32 = 0;

        // Types contribute to complexity
        score += (self.types_count as u32).min(10);

        // Fields contribute
        score += (self.fields_count as u32 / 5).min(10);

        // Nesting depth
        score += (self.max_depth as u32 * 2).min(10);

        // Optional fields indicate heterogeneity
        score += (self.optional_fields_count as u32).min(5);

        // Nested types
        score += (self.nested_types_count as u32).min(5);

        // Normalize to 1-10
        ((score as f32 / 4.0).ceil() as u8).clamp(1, 10)
    }

    /// Get complexity label with color
    pub fn complexity_label(&self) -> String {
        let score = self.complexity_score();
        match score {
            1..=3 => "Simple".green().to_string(),
            4..=6 => "Moderate".yellow().to_string(),
            7..=9 => "Complex".bright_red().to_string(),
            10 => "Very Complex".red().bold().to_string(),
            _ => "Unknown".dimmed().to_string(),
        }
    }
}

/// Calculate the nesting depth of a field type
fn calculate_type_depth(field_type: &FieldType) -> usize {
    // Use AST helper methods to resolve "unused method" warnings
    if field_type.is_primitive() {
        return 0;
    }

    if field_type.is_reference() {
        return 1;
    }

    if let Some(inner) = field_type.inner_type() {
        // Optional doesn't add to depth count for complexity
        if matches!(field_type, FieldType::Optional(_)) {
            return calculate_type_depth(inner);
        }
        return 1 + calculate_type_depth(inner);
    }

    match field_type {
        FieldType::Map(_, value) => 1 + calculate_type_depth(value),
        FieldType::Union(types) => types.iter().map(calculate_type_depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// Reporter for displaying conversion results
pub struct Reporter;

impl Reporter {
    /// Print a beautiful header
    pub fn print_header() {
        println!();
        println!(
            "{}",
            "╔═════════════════════════════════════════════════════════╗".bright_magenta()
        );
        println!(
            "{}",
            "║                                                         ║".bright_magenta()
        );
        println!(
            "{}  {}           {}",
            "║".bright_magenta(),
            "🧪 ALCHEMIST - Type Transformation Complete"
                .bright_cyan()
                .bold(),
            "║".bright_magenta()
        );
        println!(
            "{}",
            "║                                                         ║".bright_magenta()
        );
        println!(
            "{}",
            "╚═════════════════════════════════════════════════════════╝".bright_magenta()
        );
        println!();
    }

    /// Print the conversion statistics as a beautiful table
    pub fn print_stats(stats: &ConversionStats, output_format: &str) {
        Self::print_header();

        // Stats table
        println!(
            "{}",
            "┌─────────────────────────────────────────────────────────┐".bright_blue()
        );
        println!(
            "{}  {}                               {}",
            "│".bright_blue(),
            "📊 Conversion Statistics".bright_white().bold(),
            "│".bright_blue()
        );
        println!(
            "{}",
            "├─────────────────────────────────────────────────────────┤".bright_blue()
        );

        // Time elapsed
        let time_ms = stats.duration.as_secs_f64() * 1000.0;
        let time_display = if time_ms < 1.0 {
            format!("{:.3} ms", time_ms).green().to_string()
        } else if time_ms < 100.0 {
            format!("{:.2} ms", time_ms).green().to_string()
        } else if time_ms < 1000.0 {
            format!("{:.1} ms", time_ms).yellow().to_string()
        } else {
            format!("{:.2} s", time_ms / 1000.0).red().to_string()
        };
        Self::print_row("⏱️  Time Elapsed", &time_display.to_string());

        // Output format
        let format_icon = match output_format {
            "rust" | "Rust" => "🦀",
            "typescript" | "TypeScript" => "📘",
            "zod" | "Zod" => "🛡️",
            _ => "📄",
        };
        Self::print_row(
            &format!("{}  Output Format", format_icon),
            &output_format.bright_cyan().to_string(),
        );

        println!(
            "{}",
            "├─────────────────────────────────────────────────────────┤".bright_blue()
        );

        // Types generated
        Self::print_row(
            "📦 Types Generated",
            &stats.types_count.to_string().bright_yellow().to_string(),
        );

        // Fields analyzed
        Self::print_row(
            "📝 Fields Analyzed",
            &stats.fields_count.to_string().bright_yellow().to_string(),
        );

        // Optional fields
        if stats.optional_fields_count > 0 {
            Self::print_row(
                "❓ Optional Fields",
                &stats
                    .optional_fields_count
                    .to_string()
                    .bright_magenta()
                    .to_string(),
            );
        }

        // Array fields
        if stats.array_fields_count > 0 {
            Self::print_row(
                "📚 Array Fields",
                &stats
                    .array_fields_count
                    .to_string()
                    .bright_cyan()
                    .to_string(),
            );
        }

        // Nested types
        if stats.nested_types_count > 0 {
            Self::print_row(
                "🔗 Nested Types",
                &stats
                    .nested_types_count
                    .to_string()
                    .bright_white()
                    .to_string(),
            );
        }

        println!(
            "{}",
            "├─────────────────────────────────────────────────────────┤".bright_blue()
        );

        // Complexity
        let complexity_bar = Self::complexity_bar(stats.complexity_score());
        Self::print_row(
            "🎯 Complexity",
            &format!("{} {}", complexity_bar, stats.complexity_label()),
        );

        // Max depth
        Self::print_row(
            "📐 Max Nesting Depth",
            &stats.max_depth.to_string().dimmed().to_string(),
        );

        println!(
            "{}",
            "├─────────────────────────────────────────────────────────┤".bright_blue()
        );

        // Sizes
        Self::print_row("📥 Input Size", &Self::format_bytes(stats.input_size));
        Self::print_row("📤 Output Size", &Self::format_bytes(stats.output_size));

        // Compression ratio
        if stats.input_size > 0 {
            let ratio = stats.output_size as f64 / stats.input_size as f64;
            let ratio_str = format!("{:.1}x", ratio);
            let colored = if ratio < 1.0 {
                ratio_str.green().to_string()
            } else if ratio < 2.0 {
                ratio_str.yellow().to_string()
            } else {
                ratio_str.bright_red().to_string()
            };
            Self::print_row("📊 Size Ratio", &colored);
        }

        println!(
            "{}",
            "└─────────────────────────────────────────────────────────┘".bright_blue()
        );
        println!();
    }

    /// Print types summary
    pub fn print_types_summary(schema: &Schema) {
        println!(
            "{}",
            "┌─────────────────────────────────────────────────────────┐".bright_green()
        );
        println!(
            "{}  {}                                     {}",
            "│".bright_green(),
            "📋 Generated Types".bright_white().bold(),
            "│".bright_green()
        );
        println!(
            "{}",
            "├─────────────────────────────────────────────────────────┤".bright_green()
        );

        // Use root_type() to verify root existence (activates unused method)
        let root_name = schema
            .root_type()
            .map(|t| &t.name)
            .unwrap_or(&schema.root_name);

        for type_def in &schema.types {
            let optional_count = type_def.fields.iter().filter(|f| f.optional).count();
            let fields_info = if optional_count > 0 {
                format!(
                    "{} fields ({} optional)",
                    type_def.fields.len().to_string().bright_yellow(),
                    optional_count.to_string().bright_magenta()
                )
            } else {
                format!(
                    "{} fields",
                    type_def.fields.len().to_string().bright_yellow()
                )
            };

            let icon = if &type_def.name == root_name {
                "🌟"
            } else {
                "  "
            };

            // Calculate padding
            let info_len = strip_ansi_len(&fields_info);
            let padding = 29_usize.saturating_sub(info_len);

            println!(
                "{}  {} {:<20} → {}{}{}",
                "│".bright_green(),
                icon,
                type_def.name.bright_cyan().bold(),
                fields_info,
                " ".repeat(padding),
                "│".bright_green()
            );
        }

        println!(
            "{}",
            "└─────────────────────────────────────────────────────────┘".bright_green()
        );
        println!();
    }

    /// Print success message
    pub fn print_success(output_path: Option<&str>) {
        match output_path {
            Some(path) => {
                println!(
                    "  {} {} {}",
                    "✅".green(),
                    "Output written to:".bright_white(),
                    path.bright_cyan().underline()
                );
            }
            None => {
                println!(
                    "  {} {}",
                    "✅".green(),
                    "Output written to stdout".bright_white()
                );
            }
        }
        println!();
    }

    /// Print error message
    pub fn print_error(message: &str) {
        println!();
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════╗".red()
        );
        println!(
            "{}  {} {:<51} {}",
            "║".red(),
            "❌".red(),
            "Error".bright_red().bold(),
            "║".red()
        );
        println!(
            "{}",
            "╠═══════════════════════════════════════════════════════════╣".red()
        );

        // Wrap long messages
        for line in textwrap(message, 55) {
            println!("{}  {:<55} {}", "║".red(), line, "║".red());
        }

        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════╝".red()
        );
        println!();
    }

    /// Print a single row in the table
    fn print_row(label: &str, value: &str) {
        let target_label_width: usize = 24;
        let label_visible_len = strip_ansi_len(label);
        let mut label_padding = target_label_width.saturating_sub(label_visible_len);

        // Manual fix for Time Elapsed emoji width inconsistency
        if label.contains("Time") {
            label_padding += 2;
        }

        let value_visible_len = strip_ansi_len(value);
        let mut total_width: usize = 57; // Total inner width available

        // Manual fix for right border alignment on Time row
        if label.contains("Time") {
            total_width += 2;
        }

        // Calculate inner usage to determine final padding needed to reach the right border
        let inner_used = 2 + label_visible_len + label_padding + 1 + value_visible_len;
        let final_padding = total_width.saturating_sub(inner_used);

        println!(
            "{}  {}{}{}{}{}",
            "│".bright_blue(),
            label.dimmed(),
            " ".repeat(label_padding),
            value,
            " ".repeat(final_padding),
            "│".bright_blue()
        );
    }

    /// Create a visual complexity bar
    fn complexity_bar(score: u8) -> String {
        let filled = score as usize;
        let empty = 10 - filled;

        let bar: String = (0..filled)
            .map(|i| {
                if i < 3 {
                    "█".green().to_string()
                } else if i < 6 {
                    "█".yellow().to_string()
                } else if i < 9 {
                    "█".bright_red().to_string()
                } else {
                    "█".red().to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("");

        let empty_bar = "░".repeat(empty).dimmed().to_string();

        format!("[{}{}]", bar, empty_bar)
    }

    /// Format bytes in human-readable format
    fn format_bytes(bytes: usize) -> String {
        if bytes < 1024 {
            format!("{} B", bytes).dimmed().to_string()
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
                .dimmed()
                .to_string()
        } else {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
                .dimmed()
                .to_string()
        }
    }
}

/// Simple text wrapping
fn textwrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Get length of string without ANSI codes
fn strip_ansi_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }

    len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Field, TypeDef};

    #[test]
    fn test_complexity_score() {
        // Simple schema
        let mut schema = Schema::new("Simple");
        let mut type_def = TypeDef::new("Simple");
        type_def.add_field(Field::new("name", FieldType::String));
        schema.add_type(type_def);

        let stats = ConversionStats::from_schema(&schema, Duration::from_millis(10), 100, 200);
        assert!(
            stats.complexity_score() <= 3,
            "Simple schema should have low complexity"
        );
    }

    #[test]
    fn test_format_bytes() {
        assert!(Reporter::format_bytes(500).contains("500 B"));
        assert!(Reporter::format_bytes(2048).contains("KB"));
        assert!(Reporter::format_bytes(2 * 1024 * 1024).contains("MB"));
    }
}
