/// Type mapping module for converting Python types to Rust types
use std::collections::HashMap;

/// Represents a Rust type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustType {
    /// Primitive types
    String,
    I32,
    I64,
    F32,
    F64,
    Bool,

    /// Collection types
    Vec(Box<RustType>),
    HashMap(Box<RustType>, Box<RustType>),

    /// Option type
    Option(Box<RustType>),

    /// Generic JSON value (fallback)
    JsonValue,

    /// Custom type (struct name)
    Custom(String),

    /// Unit type
    Unit,
}

impl RustType {
    /// Convert to Rust type string
    pub fn to_rust_string(&self) -> String {
        match self {
            RustType::String => "String".to_string(),
            RustType::I32 => "i32".to_string(),
            RustType::I64 => "i64".to_string(),
            RustType::F32 => "f32".to_string(),
            RustType::F64 => "f64".to_string(),
            RustType::Bool => "bool".to_string(),
            RustType::Vec(inner) => format!("Vec<{}>", inner.to_rust_string()),
            RustType::HashMap(k, v) => {
                format!("HashMap<{}, {}>", k.to_rust_string(), v.to_rust_string())
            }
            RustType::Option(inner) => format!("Option<{}>", inner.to_rust_string()),
            RustType::JsonValue => "serde_json::Value".to_string(),
            RustType::Custom(name) => name.clone(),
            RustType::Unit => "()".to_string(),
        }
    }

    /// Get required imports for this type
    pub fn required_imports(&self) -> Vec<&'static str> {
        match self {
            RustType::HashMap(_, _) => vec!["std::collections::HashMap"],
            RustType::JsonValue => vec!["serde_json::Value"],
            RustType::Vec(inner) | RustType::Option(inner) => inner.required_imports(),
            _ => vec![],
        }
    }
}

/// Map Python type string to Rust type
pub fn map_python_type(py_type: &str) -> RustType {
    // Handle simple types
    match py_type {
        "str" => RustType::String,
        "int" => RustType::I64,
        "float" => RustType::F64,
        "bool" => RustType::Bool,
        "None" | "NoneType" => RustType::Unit,
        "Any" | "object" => RustType::JsonValue,
        _ => {
            // Handle complex types
            if py_type.starts_with("list[") || py_type.starts_with("List[") {
                parse_list_type(py_type)
            } else if py_type.starts_with("dict[") || py_type.starts_with("Dict[") {
                parse_dict_type(py_type)
            } else if py_type.starts_with("Optional[") {
                parse_optional_type(py_type)
            } else if py_type.contains("|") {
                parse_union_type(py_type)
            } else {
                // Custom type
                RustType::Custom(py_type.to_string())
            }
        }
    }
}

/// Parse list[T] or List[T] type
fn parse_list_type(py_type: &str) -> RustType {
    if let Some(inner) = extract_generic_param(py_type) {
        let inner_rust_type = map_python_type(&inner);
        RustType::Vec(Box::new(inner_rust_type))
    } else {
        // Fallback to Vec<JsonValue>
        RustType::Vec(Box::new(RustType::JsonValue))
    }
}

/// Parse dict[K, V] or Dict[K, V] type
fn parse_dict_type(py_type: &str) -> RustType {
    if let Some(params) = extract_generic_param(py_type) {
        let parts: Vec<&str> = params.split(',').map(|s| s.trim()).collect();

        if parts.len() == 2 {
            let key_type = map_python_type(parts[0]);
            let value_type = map_python_type(parts[1]);
            RustType::HashMap(Box::new(key_type), Box::new(value_type))
        } else {
            // Fallback to HashMap<String, JsonValue>
            RustType::HashMap(Box::new(RustType::String), Box::new(RustType::JsonValue))
        }
    } else {
        // Fallback to HashMap<String, JsonValue>
        RustType::HashMap(Box::new(RustType::String), Box::new(RustType::JsonValue))
    }
}

/// Parse Optional[T] type
fn parse_optional_type(py_type: &str) -> RustType {
    if let Some(inner) = extract_generic_param(py_type) {
        let inner_rust_type = map_python_type(&inner);
        RustType::Option(Box::new(inner_rust_type))
    } else {
        RustType::Option(Box::new(RustType::JsonValue))
    }
}

/// Parse union types (T | U)
fn parse_union_type(py_type: &str) -> RustType {
    let parts: Vec<&str> = py_type.split('|').map(|s| s.trim()).collect();

    // Check if it's Optional (X | None)
    if parts.contains(&"None") || parts.contains(&"NoneType") {
        let non_none_parts: Vec<&str> = parts
            .iter()
            .filter(|&p| p != &"None" && p != &"NoneType")
            .copied()
            .collect();

        if non_none_parts.len() == 1 {
            let inner_type = map_python_type(non_none_parts[0]);
            return RustType::Option(Box::new(inner_type));
        }
    }

    // For other unions, use JsonValue as fallback
    RustType::JsonValue
}

/// Extract generic parameter from Type[Param] notation
fn extract_generic_param(s: &str) -> Option<String> {
    let start = s.find('[')?;
    let end = s.rfind(']')?;

    if start < end {
        Some(s[start + 1..end].to_string())
    } else {
        None
    }
}

/// Type mapping configuration
pub struct TypeMapper {
    /// Custom type mappings
    custom_mappings: HashMap<String, RustType>,
}

impl TypeMapper {
    /// Create a new type mapper
    pub fn new() -> Self {
        TypeMapper {
            custom_mappings: HashMap::new(),
        }
    }

    /// Add a custom type mapping
    pub fn add_mapping(&mut self, python_type: String, rust_type: RustType) {
        self.custom_mappings.insert(python_type, rust_type);
    }

    /// Map a Python type to Rust, checking custom mappings first
    pub fn map_type(&self, py_type: &str) -> RustType {
        if let Some(rust_type) = self.custom_mappings.get(py_type) {
            rust_type.clone()
        } else {
            map_python_type(py_type)
        }
    }
}

impl Default for TypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        assert_eq!(map_python_type("str"), RustType::String);
        assert_eq!(map_python_type("int"), RustType::I64);
        assert_eq!(map_python_type("float"), RustType::F64);
        assert_eq!(map_python_type("bool"), RustType::Bool);
    }

    #[test]
    fn test_list_types() {
        let result = map_python_type("list[str]");
        assert_eq!(result, RustType::Vec(Box::new(RustType::String)));

        let result = map_python_type("List[int]");
        assert_eq!(result, RustType::Vec(Box::new(RustType::I64)));
    }

    #[test]
    fn test_dict_types() {
        let result = map_python_type("dict[str, int]");
        assert_eq!(
            result,
            RustType::HashMap(Box::new(RustType::String), Box::new(RustType::I64))
        );
    }

    #[test]
    fn test_optional_types() {
        let result = map_python_type("Optional[str]");
        assert_eq!(result, RustType::Option(Box::new(RustType::String)));
    }

    #[test]
    fn test_union_types() {
        let result = map_python_type("str | None");
        assert_eq!(result, RustType::Option(Box::new(RustType::String)));
    }

    #[test]
    fn test_nested_types() {
        let result = map_python_type("list[dict[str, int]]");
        let expected = RustType::Vec(Box::new(RustType::HashMap(
            Box::new(RustType::String),
            Box::new(RustType::I64),
        )));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_rust_string() {
        let t = RustType::Vec(Box::new(RustType::String));
        assert_eq!(t.to_rust_string(), "Vec<String>");

        let t = RustType::HashMap(Box::new(RustType::String), Box::new(RustType::I64));
        assert_eq!(t.to_rust_string(), "HashMap<String, i64>");

        let t = RustType::Option(Box::new(RustType::String));
        assert_eq!(t.to_rust_string(), "Option<String>");
    }

    #[test]
    fn test_custom_mapper() {
        let mut mapper = TypeMapper::new();
        mapper.add_mapping("UserId".to_string(), RustType::Custom("UserId".to_string()));

        assert_eq!(
            mapper.map_type("UserId"),
            RustType::Custom("UserId".to_string())
        );

        // Fallback to default mapping
        assert_eq!(mapper.map_type("str"), RustType::String);
    }
}
