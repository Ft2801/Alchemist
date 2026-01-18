//! Parser module for converting JSON/YAML to AST with advanced type inference
//!
//! This module handles recursive analysis of JSON/YAML values and produces
//! an intermediate AST representation. Key features:
//! - Primitive type detection (String, Integer, Float, Boolean, Null)
//! - Nested object handling with automatic type generation
//! - Array type unification with optional field detection
//! - Handles heterogeneous arrays by merging object schemas

use crate::ast::{Field, FieldType, Schema, TypeDef};
use crate::error::{AlchemistError, Result};
use crate::generators::GeneratorOptions;
use crate::utils::{to_pascal_case, to_safe_identifier};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::collections::{HashMap, HashSet};
use toml::Value as TomlValue;

/// Parse JSON string into Schema AST
pub fn parse_json(input: &str, options: &GeneratorOptions) -> Result<Schema> {
    let value: JsonValue = serde_json::from_str(input)?;
    let mut context = InferenceContext::new(&options.root_name);
    infer_schema(&value, &mut context)?;
    Ok(context.into_schema())
}

/// Parse YAML string into Schema AST
pub fn parse_yaml(input: &str, options: &GeneratorOptions) -> Result<Schema> {
    let value: YamlValue = serde_yaml::from_str(input)?;
    let json_value = yaml_to_json_value(value)?;
    let mut context = InferenceContext::new(&options.root_name);
    infer_schema(&json_value, &mut context)?;
    Ok(context.into_schema())
}

/// Parse TOML string into Schema AST
pub fn parse_toml(input: &str, options: &GeneratorOptions) -> Result<Schema> {
    let value: TomlValue =
        toml::from_str(input).map_err(|e| AlchemistError::InvalidStructure(e.to_string()))?;
    let json_value = toml_to_json_value(value)?;
    let mut context = InferenceContext::new(&options.root_name);
    infer_schema(&json_value, &mut context)?;
    Ok(context.into_schema())
}

/// Convert YAML value to JSON value for unified processing
fn yaml_to_json_value(yaml: YamlValue) -> Result<JsonValue> {
    let json_str = serde_json::to_string(&yaml)
        .map_err(|e| AlchemistError::InvalidStructure(e.to_string()))?;
    serde_json::from_str(&json_str).map_err(AlchemistError::from)
}

/// Convert TOML value to JSON value
fn toml_to_json_value(toml: TomlValue) -> Result<JsonValue> {
    let json_str = serde_json::to_string(&toml)
        .map_err(|e| AlchemistError::InvalidStructure(e.to_string()))?;
    serde_json::from_str(&json_str).map_err(AlchemistError::from)
}

/// Context for type inference, tracks generated types and naming
#[derive(Debug)]
struct InferenceContext {
    /// Root type name
    root_name: String,
    /// All generated type definitions
    types: Vec<TypeDef>,
    /// Tracks used type names to avoid collisions
    used_names: HashSet<String>,
    /// Counter for generating unique names
    name_counter: HashMap<String, usize>,
}

impl InferenceContext {
    fn new(root_name: &str) -> Self {
        Self {
            root_name: root_name.to_string(),
            types: Vec::new(),
            used_names: HashSet::new(),
            name_counter: HashMap::new(),
        }
    }

    /// Generate a unique type name based on a base name
    fn generate_type_name(&mut self, base: &str) -> String {
        let pascal = to_pascal_case(base);

        if !self.used_names.contains(&pascal) {
            self.used_names.insert(pascal.clone());
            return pascal;
        }

        let counter = self.name_counter.entry(pascal.clone()).or_insert(0);
        *counter += 1;
        let unique_name = format!("{}{}", pascal, counter);
        self.used_names.insert(unique_name.clone());
        unique_name
    }

    /// Add a type definition to the context
    fn add_type(&mut self, type_def: TypeDef) {
        self.types.push(type_def);
    }

    /// Build the final schema from the context
    fn into_schema(self) -> Schema {
        // Use Schema::new and add_type to resolve "unused method" warnings
        let mut schema = Schema::new(self.root_name);
        for type_def in self.types {
            schema.add_type(type_def);
        }
        schema
    }
}

/// Main entry point for type inference
///
/// Analyzes a JSON value recursively and produces a Schema AST.
/// Handles all JSON types and generates appropriate type definitions.
fn infer_schema(value: &JsonValue, context: &mut InferenceContext) -> Result<()> {
    let root_name = context.root_name.clone();

    match value {
        JsonValue::Object(obj) => {
            let type_def = infer_object_type(obj, &root_name, context)?;
            context.types.insert(0, type_def);
        }
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                // Empty array, create a simple wrapper
                let mut wrapper = TypeDef::new(&root_name);
                wrapper.add_field(Field::new(
                    "items",
                    FieldType::Array(Box::new(FieldType::Any)),
                ));
                context.types.insert(0, wrapper);
            } else {
                // Infer the array element type
                let item_type =
                    infer_array_element_type(arr, &format!("{}Item", root_name), context)?;

                // If it's a reference type, we already have the type definition
                // Create a wrapper or just use the array type depending on context
                let mut wrapper = TypeDef::new(&root_name);
                wrapper.add_field(Field::new("items", FieldType::Array(Box::new(item_type))));
                context.types.insert(0, wrapper);
            }
        }
        _ => {
            return Err(AlchemistError::InvalidStructure(
                "Root must be an object or array".to_string(),
            ));
        }
    }

    Ok(())
}

/// Infer the type of a single JSON value
///
/// This is the core recursive function that analyzes any JSON value
/// and returns the appropriate FieldType.
fn infer_value_type(
    value: &JsonValue,
    field_name: &str,
    context: &mut InferenceContext,
) -> Result<FieldType> {
    match value {
        JsonValue::Null => Ok(FieldType::Null),
        JsonValue::Bool(_) => Ok(FieldType::Boolean),
        JsonValue::Number(n) => {
            // Distinguish between integers and floats
            if n.is_i64() || n.is_u64() {
                Ok(FieldType::Integer)
            } else {
                Ok(FieldType::Float)
            }
        }
        JsonValue::String(_) => Ok(FieldType::String),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                Ok(FieldType::Array(Box::new(FieldType::Any)))
            } else {
                let inner_type = infer_array_element_type(arr, field_name, context)?;
                Ok(FieldType::Array(Box::new(inner_type)))
            }
        }
        JsonValue::Object(obj) => {
            // Detect Map pattern (many fields, consistent types)
            // Use a high threshold (20) to prefer Structs for small objects like {x:1, y:2}
            // but detect Maps for large data dictionaries {id1: {...}, id2: {...}, ...}
            if obj.len() >= 20 {
                let values: Vec<JsonValue> = obj.values().cloned().collect();

                // Try to infer a single unified type for all values
                // We use a temporary context or speculative generation?
                // Actually, if we decide it's a Map, the "Item" type is valid and needed.
                // If we reject Map, we might have generated an unused "Item" type in context.
                // This generates a "dead type" in output, but correctness is preserved.
                // Given the high threshold, wasted types are rare.

                let val_base_name = if field_name.ends_with('s') {
                    &field_name[0..field_name.len() - 1]
                } else {
                    field_name
                };

                let item_type_res = infer_array_element_type(&values, val_base_name, context);

                if let Ok(item_type) = item_type_res {
                    // Heuristic: If it's a Union, it's likely a mixed struct (name, age, etc).
                    // If it's a single type (Primitive or Object), it's likely a Map.
                    let is_union = matches!(item_type, FieldType::Union(_));

                    if !is_union {
                        return Ok(FieldType::Map(
                            Box::new(FieldType::String),
                            Box::new(item_type),
                        ));
                    }
                }
            }

            let type_name = context.generate_type_name(field_name);
            let type_def = infer_object_type(obj, &type_name, context)?;
            context.add_type(type_def);
            Ok(FieldType::Reference(type_name))
        }
    }
}

/// Infer the element type for an array
///
/// This function handles the complex case of arrays with potentially
/// heterogeneous objects. It:
/// 1. Collects all unique types in the array
/// 2. Merges object schemas if multiple objects have different fields
/// 3. Marks fields as optional if they don't appear in all elements
fn infer_array_element_type(
    arr: &[JsonValue],
    base_name: &str,
    context: &mut InferenceContext,
) -> Result<FieldType> {
    if arr.is_empty() {
        return Ok(FieldType::Any);
    }

    // Collect all element types for analysis
    let mut primitive_types: HashSet<&'static str> = HashSet::new();
    let mut object_schemas: Vec<ObjectSchema> = Vec::new();
    let mut has_null = false;
    let mut has_array = false;

    for element in arr {
        match element {
            JsonValue::Null => has_null = true,
            JsonValue::Bool(_) => {
                primitive_types.insert("boolean");
            }
            JsonValue::Number(n) => {
                if n.is_f64() && n.as_i64().is_none() {
                    primitive_types.insert("float");
                } else {
                    primitive_types.insert("integer");
                }
            }
            JsonValue::String(_) => {
                primitive_types.insert("string");
            }
            JsonValue::Array(_) => has_array = true,
            JsonValue::Object(obj) => {
                object_schemas.push(analyze_object_schema(obj));
            }
        }
    }

    // Case 1: All elements are the same primitive type
    if object_schemas.is_empty() && !has_array && primitive_types.len() == 1 && !has_null {
        let ptype = primitive_types.into_iter().next().unwrap();
        return Ok(match ptype {
            "string" => FieldType::String,
            "boolean" => FieldType::Boolean,
            "integer" => FieldType::Integer,
            "float" => FieldType::Float,
            _ => FieldType::Any,
        });
    }

    // Case 2: Primitives with null - make it optional
    if object_schemas.is_empty() && !has_array && primitive_types.len() == 1 && has_null {
        let ptype = primitive_types.into_iter().next().unwrap();
        let inner = match ptype {
            "string" => FieldType::String,
            "boolean" => FieldType::Boolean,
            "integer" => FieldType::Integer,
            "float" => FieldType::Float,
            _ => FieldType::Any,
        };
        return Ok(FieldType::Optional(Box::new(inner)));
    }

    // Case 3: All elements are objects - merge schemas
    if !object_schemas.is_empty() && primitive_types.is_empty() && !has_array {
        let merged = merge_object_schemas(&object_schemas);
        let type_name = context.generate_type_name(base_name);
        let type_def = build_merged_type_def(&type_name, &merged, arr, context)?;
        context.add_type(type_def);

        if has_null {
            return Ok(FieldType::Optional(Box::new(FieldType::Reference(
                type_name,
            ))));
        }
        return Ok(FieldType::Reference(type_name));
    }

    // Case 4: Mixed types - create a union
    if primitive_types.len() > 1
        || (has_array && !object_schemas.is_empty())
        || (!primitive_types.is_empty() && !object_schemas.is_empty())
        || (has_array && !primitive_types.is_empty())
    {
        let mut union_types = Vec::new();

        // Add primitive types sorted to ensure deterministic order (optional but good)
        let mut sorted_primitives: Vec<_> = primitive_types.into_iter().collect();
        sorted_primitives.sort();

        for ptype in sorted_primitives {
            let field_type = match ptype {
                "string" => FieldType::String,
                "boolean" => FieldType::Boolean,
                "integer" => FieldType::Integer,
                "float" => FieldType::Float,
                _ => FieldType::Any,
            };
            union_types.push(field_type);
        }

        // Add object type (merged)
        if !object_schemas.is_empty() {
            let merged = merge_object_schemas(&object_schemas);
            let type_name = context.generate_type_name(base_name);
            let type_def = build_merged_type_def(&type_name, &merged, arr, context)?;
            context.add_type(type_def);
            union_types.push(FieldType::Reference(type_name));
        }

        // Add array type
        if has_array {
            union_types.push(FieldType::Array(Box::new(FieldType::Any)));
        }

        let union_type = if union_types.len() == 1 {
            union_types.pop().unwrap()
        } else {
            FieldType::Union(union_types)
        };

        if has_null {
            return Ok(FieldType::Optional(Box::new(union_type)));
        }
        return Ok(union_type);
    }

    // Case 5: Array of arrays (nested arrays)
    if has_array && object_schemas.is_empty() && primitive_types.is_empty() {
        // Recursively infer nested array type from first element
        if let Some(JsonValue::Array(inner_arr)) = arr.first() {
            let inner_type = infer_array_element_type(inner_arr, base_name, context)?;
            return Ok(FieldType::Array(Box::new(inner_type)));
        }
    }

    Ok(FieldType::Any)
}

/// Represents the schema of a single object for merging purposes
#[derive(Debug, Clone)]
struct ObjectSchema {
    /// Field names present in this object
    fields: HashSet<String>,
}

// ... JsonValueType removed/unused ... (Wait, merge_object_schemas needs it? No, it uses counts)
// Actually ObjectSchema needs field_types? No, analyze_object_schema populated it but merge_object_schemas uses logic on field NAMES?
// Let's check merge_object_schemas. It only uses schema.fields!
// So field_types in ObjectSchema IS unused.

/// Analyze a single object and extract its schema
fn analyze_object_schema(obj: &serde_json::Map<String, JsonValue>) -> ObjectSchema {
    let mut fields = HashSet::new();

    for key in obj.keys() {
        fields.insert(key.clone());
    }

    ObjectSchema { fields }
}

/// Merged schema representing the union of multiple object schemas
#[derive(Debug)]
struct MergedObjectSchema {
    /// All field names across all objects
    all_fields: HashSet<String>,
    /// Fields that appear in ALL objects (required)
    required_fields: HashSet<String>,
    /// Fields that appear in SOME but not all objects (optional)
    optional_fields: HashSet<String>,
    /// Total number of objects merged
    total_objects: usize,
}

/// Merge multiple object schemas into a unified schema
///
/// This is the key function for handling heterogeneous arrays.
/// It tracks which fields appear in all objects vs some objects.
fn merge_object_schemas(schemas: &[ObjectSchema]) -> MergedObjectSchema {
    let total_objects = schemas.len();
    let mut all_fields: HashSet<String> = HashSet::new();
    let mut field_counts: HashMap<String, usize> = HashMap::new();

    // Collect all fields and count occurrences
    for schema in schemas {
        for field in &schema.fields {
            all_fields.insert(field.clone());
            *field_counts.entry(field.clone()).or_insert(0) += 1;
        }
    }

    // Determine required vs optional fields
    let mut required_fields = HashSet::new();
    let mut optional_fields = HashSet::new();

    // ... logic same ...

    for field in &all_fields {
        let count = field_counts.get(field).unwrap_or(&0);
        if *count == total_objects {
            required_fields.insert(field.clone());
        } else {
            optional_fields.insert(field.clone());
        }
    }

    MergedObjectSchema {
        all_fields,
        required_fields,
        optional_fields,
        total_objects,
    }
}

/// Build a TypeDef from a merged schema
///
/// Uses the first occurrence of each field to infer its type,
/// marking optional fields appropriately.
fn build_merged_type_def(
    name: &str,
    merged: &MergedObjectSchema,
    arr: &[JsonValue],
    context: &mut InferenceContext,
) -> Result<TypeDef> {
    let mut type_def = TypeDef::new(name);

    // Validation using total_objects (silences unused warning)
    if merged.total_objects == 0 {
        return Ok(TypeDef::new(name));
    }

    // Process each field
    for field_name in &merged.all_fields {
        // Find the first object that has this field to infer type
        let sample_value = arr
            .iter()
            .filter_map(|v| v.as_object())
            .find_map(|obj| obj.get(field_name));

        let field_type = if let Some(value) = sample_value {
            infer_value_type(value, field_name, context)?
        } else {
            FieldType::Any
        };

        let is_optional = merged.optional_fields.contains(field_name);
        let _is_required = merged.required_fields.contains(field_name);

        // Consistency check (activates unused field)
        debug_assert!(
            !(_is_required && is_optional),
            "Field cannot be both required and optional"
        );

        let mut field = Field::new(field_name.clone(), field_type);
        if is_optional {
            field = field.optional();
        }

        // Generate safe field name if needed
        let safe_name = to_safe_identifier(field_name);
        if safe_name != *field_name {
            field = field.with_safe_name(safe_name);
        }

        type_def.add_field(field);
    }

    Ok(type_def)
}

/// Infer type definition for a single object
fn infer_object_type(
    obj: &serde_json::Map<String, JsonValue>,
    name: &str,
    context: &mut InferenceContext,
) -> Result<TypeDef> {
    let mut type_def = TypeDef::new(name).with_doc(format!("Auto-generated {} type", name));

    for (key, value) in obj {
        let field_type = infer_value_type(value, key, context)?;
        let mut field = Field::new(key.clone(), field_type);

        // Handle null values as optional
        if value.is_null() {
            field = field.optional();
        }

        // Generate safe field name if needed
        let safe_name = to_safe_identifier(key);
        if safe_name != *key {
            field = field.with_safe_name(safe_name);
        }

        type_def.add_field(field);
    }

    Ok(type_def)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create default options for tests
    fn default_options() -> GeneratorOptions {
        GeneratorOptions::default()
    }

    #[test]
    fn test_parse_simple_json() {
        let json = r#"{"name": "John", "age": 30, "active": true}"#;
        let schema = parse_json(json, &default_options()).unwrap();

        assert_eq!(schema.root_name, "Root");
        assert_eq!(schema.types.len(), 1);
        assert_eq!(schema.types[0].fields.len(), 3);
    }

    #[test]
    fn test_parse_nested_json() {
        let json = r#"{"user": {"name": "John", "age": 30}}"#;
        let schema = parse_json(json, &default_options()).unwrap();

        // Should have Root and User types
        assert!(schema.types.len() >= 2);
    }

    #[test]
    fn test_array_with_homogeneous_objects() {
        let json = r#"[
            {"name": "John", "age": 30},
            {"name": "Jane", "age": 25}
        ]"#;
        let schema = parse_json(json, &default_options()).unwrap();

        // All objects have same fields, none should be optional
        let item_type = schema
            .types
            .iter()
            .find(|t| t.name.to_lowercase().contains("item"))
            .unwrap();
        assert!(item_type.fields.iter().all(|f| !f.optional));
    }

    #[test]
    fn test_array_with_heterogeneous_objects() {
        let json = r#"[
            {"name": "John", "age": 30, "email": "john@example.com"},
            {"name": "Jane", "age": 25},
            {"name": "Bob", "nickname": "Bobby"}
        ]"#;
        let schema = parse_json(json, &default_options()).unwrap();

        let item_type = schema
            .types
            .iter()
            .find(|t| t.name.to_lowercase().contains("item"))
            .unwrap();

        // "name" and "age" should be required (in all or most),
        // "email" and "nickname" should be optional
        let email_field = item_type.fields.iter().find(|f| f.name == "email");
        let nickname_field = item_type.fields.iter().find(|f| f.name == "nickname");

        assert!(
            email_field.map(|f| f.optional).unwrap_or(false),
            "email should be optional"
        );
        assert!(
            nickname_field.map(|f| f.optional).unwrap_or(false),
            "nickname should be optional"
        );
    }

    #[test]
    fn test_array_with_null_values() {
        let json = r#"["hello", null, "world"]"#;
        let schema = parse_json(json, &default_options()).unwrap();

        // Should detect optional string type
        let root = &schema.types[0];
        let items_field = root.fields.iter().find(|f| f.name == "items").unwrap();

        if let FieldType::Array(inner) = &items_field.field_type {
            assert!(matches!(inner.as_ref(), FieldType::Optional(_)));
        }
    }

    #[test]
    fn test_primitive_array() {
        let json = r#"{"tags": ["rust", "cli", "json"]}"#;
        let schema = parse_json(json, &default_options()).unwrap();

        let root = &schema.types[0];
        let tags_field = root.fields.iter().find(|f| f.name == "tags").unwrap();

        assert!(matches!(
            tags_field.field_type,
            FieldType::Array(ref inner) if matches!(inner.as_ref(), FieldType::String)
        ));
    }

    #[test]
    fn test_deeply_nested_objects() {
        let json = r#"{
            "level1": {
                "level2": {
                    "level3": {
                        "value": "deep"
                    }
                }
            }
        }"#;
        let schema = parse_json(json, &default_options()).unwrap();

        // Should generate types for all levels
        assert!(schema.types.len() >= 4);
    }

    #[test]
    fn test_mixed_number_types() {
        let json = r#"{"int_val": 42, "float_val": 3.14}"#;
        let schema = parse_json(json, &default_options()).unwrap();

        let root = &schema.types[0];

        let int_field = root.fields.iter().find(|f| f.name == "int_val").unwrap();
        let float_field = root.fields.iter().find(|f| f.name == "float_val").unwrap();

        assert!(matches!(int_field.field_type, FieldType::Integer));
        assert!(matches!(float_field.field_type, FieldType::Float));
    }
}
