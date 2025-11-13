use crate::syntax::Expr;

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

    pub(super) fn from_expr(value: &Expr) -> Self {
        match value {
            Expr::BoolLiteral(_) => Self::Bool,
            Expr::IntLiteral(_) => Self::Int,
            Expr::FloatLiteral(_) => Self::Float,
            Expr::StringLiteral(_) => Self::String,
            Expr::Tuple(items) => {
                Self::Tuple(items.iter().map(|i| Self::from_expr(&i.inner)).collect())
            }
            _ => todo!(), // コンテキストを見て式を評価して型を決める
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
