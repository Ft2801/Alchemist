//! Intermediate AST representation for parsed data structures

/// Represents a complete schema with multiple type definitions
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    /// The root type name
    pub root_name: String,
    /// All type definitions in the schema
    pub types: Vec<TypeDef>,
}

impl Schema {
    /// Create a new schema with a root name
    pub fn new(root_name: impl Into<String>) -> Self {
        Self {
            root_name: root_name.into(),
            types: Vec::new(),
        }
    }

    /// Add a type definition to the schema
    pub fn add_type(&mut self, type_def: TypeDef) {
        self.types.push(type_def);
    }

    /// Get the root type definition
    pub fn root_type(&self) -> Option<&TypeDef> {
        self.types.iter().find(|t| t.name == self.root_name)
    }
}

/// Represents a type definition (struct/interface)
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    /// Name of the type
    pub name: String,
    /// Documentation comment
    pub doc: Option<String>,
    /// Fields of the type
    pub fields: Vec<Field>,
}

impl TypeDef {
    /// Create a new type definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            doc: None,
            fields: Vec::new(),
        }
    }

    /// Add a documentation comment
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Add a field to the type
    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }
}

/// Represents a field in a type definition
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Field name (original from JSON/YAML)
    pub name: String,
    /// Sanitized field name for the target language
    pub safe_name: Option<String>,
    /// Type of the field
    pub field_type: FieldType,
    /// Whether the field is optional
    pub optional: bool,
    /// Documentation comment
    pub doc: Option<String>,
}

impl Field {
    /// Create a new field
    pub fn new(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            safe_name: None,
            field_type,
            optional: false,
            doc: None,
        }
    }

    /// Mark field as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Set a safe name for the field
    pub fn with_safe_name(mut self, safe_name: impl Into<String>) -> Self {
        self.safe_name = Some(safe_name.into());
        self
    }

    /// Get the name to use in generated code
    pub fn code_name(&self) -> &str {
        self.safe_name.as_deref().unwrap_or(&self.name)
    }
}

/// Represents the type of a field
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// String type
    String,
    /// Integer type (i64)
    Integer,
    /// Floating point type (f64)
    Float,
    /// Boolean type
    Boolean,
    /// Null type
    Null,
    /// Array of a specific type
    Array(Box<FieldType>),
    /// Optional/nullable type
    Optional(Box<FieldType>),
    /// Reference to another type definition
    Reference(String),
    /// Union of multiple types
    Union(Vec<FieldType>),
    /// Any/unknown type
    Any,
    /// Map/Record type
    Map(Box<FieldType>, Box<FieldType>),
}

impl FieldType {
    /// Check if the type is a primitive
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            FieldType::String
                | FieldType::Integer
                | FieldType::Float
                | FieldType::Boolean
                | FieldType::Null
        )
    }

    /// Check if the type is a reference to another type
    pub fn is_reference(&self) -> bool {
        matches!(self, FieldType::Reference(_))
    }

    /// Get the inner type for arrays and optionals
    pub fn inner_type(&self) -> Option<&FieldType> {
        match self {
            FieldType::Array(inner) | FieldType::Optional(inner) => Some(inner),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let mut schema = Schema::new("User");

        let mut user_type = TypeDef::new("User");
        user_type.add_field(Field::new("name", FieldType::String));
        user_type.add_field(Field::new("age", FieldType::Integer));

        schema.add_type(user_type);

        assert_eq!(schema.root_name, "User");
        assert_eq!(schema.types.len(), 1);
    }

    #[test]
    fn test_field_type_is_primitive() {
        assert!(FieldType::String.is_primitive());
        assert!(FieldType::Integer.is_primitive());
        assert!(FieldType::Float.is_primitive());
        assert!(FieldType::Boolean.is_primitive());
        assert!(!FieldType::Array(Box::new(FieldType::String)).is_primitive());
    }
}
