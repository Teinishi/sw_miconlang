use crate::syntax::LiteralValue;

#[derive(Debug)]
pub enum ValueType {
    Bool,
    Int,
    Float,
    String,
    Tuple(Vec<ValueType>),
}

impl ValueType {
    pub(super) fn from_str(type_name: &str) -> Result<Self, &str> {
        match type_name {
            "bool" => Ok(Self::Bool),
            "int" => Ok(Self::Int),
            "float" => Ok(Self::Float),
            "string" => Ok(Self::String),
            _ => Err(type_name),
        }
    }
}

impl From<&LiteralValue> for ValueType {
    fn from(value: &LiteralValue) -> Self {
        match value {
            LiteralValue::Bool(_) => Self::Bool,
            LiteralValue::Int(_) => Self::Int,
            LiteralValue::Float(_) => Self::Float,
            LiteralValue::String(_) => Self::String,
            LiteralValue::Tuple(items) => Self::Tuple(items.iter().map(|i| i.into()).collect()),
        }
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::String => write!(f, "string"),
            Self::Tuple(items) => {
                write!(f, "tuple(")?;
                let mut iter = items.iter();
                if let Some(first) = iter.next() {
                    write!(f, "{}", first)?;
                }
                for item in iter {
                    write!(f, ", {}", item)?;
                }
                write!(f, ")")
            }
        }
    }
}
