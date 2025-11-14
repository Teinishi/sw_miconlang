use crate::{lexical::Token, microcontroller::NodeType, semantic::ValueType};

use ariadne::{Color, Label};
use chumsky::error::RichPattern;
use std::ops::{Range, RangeInclusive};

#[derive(Debug)]
pub enum CompileErrorType {
    InvalidToken,
    UnexpectedToken {
        expected: String,
        found: Option<Token>,
    },
    UnknownField {
        ident: String,
    },
    InvalidAssignment,
    IncompatibleType {
        expected_types: Vec<ValueType>,
        found_type: ValueType,
    },
    OutOfBounds {
        bounds: RangeInclusive<i64>,
    },
    UnknownType {
        type_name: String,
    },
    FieldAlreadyDeclared,   // todo: 先にどこで定義されているか表示できるように
    ElementAlreadyDeclared, // todo: 先にどこで定義されているか表示できるように
    StringInLogic,
    FieldAccessOnly,
    OutputsInExpression,
    NodeDoesNotExist {
        //component: Component,
        component_str: String,
        index: usize,
    },
    IncompatibleNodeType {
        expected_type: NodeType,
        found_type: NodeType,
    },
}

impl CompileErrorType {
    pub fn unexpected_token(e: &chumsky::error::Rich<'_, Token, Range<usize>>) -> Self {
        let mut expected = String::new();
        for token in e.expected() {
            if !expected.is_empty() {
                expected += " or ";
            }
            match token {
                RichPattern::Token(t) => expected += &format!("{:?}", t),
                RichPattern::Label(t) => expected += &format!("{:}", t),
                RichPattern::Identifier(t) => expected += &format!("identifier {:?}", t),
                RichPattern::Any => expected += "anything",
                RichPattern::SomethingElse => expected += "something else",
                RichPattern::EndOfInput => expected += "end of file",
            }
        }

        Self::UnexpectedToken {
            expected,
            found: e.found().cloned(),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::InvalidToken => "Invalid Token",
            Self::UnexpectedToken { .. } => "Unexpected Token",
            Self::UnknownField { .. } => "Unknown Field",
            Self::InvalidAssignment => "Invalid Assignment",
            Self::IncompatibleType { .. } => "Incompatible Types",
            Self::OutOfBounds { .. } => "Out of Bounds",
            Self::UnknownType { .. } => "Unknown Type",
            Self::FieldAlreadyDeclared => "Field Already Declared",
            Self::ElementAlreadyDeclared => "Element Already Declared",
            Self::StringInLogic => "String in Logic",
            Self::FieldAccessOnly => "Field Access Only",
            Self::OutputsInExpression => "Outputs in Expression",
            Self::NodeDoesNotExist { .. } => "Node Does Not Exist",
            Self::IncompatibleNodeType { .. } => "Incompatible Node Type",
        }
    }

    pub(super) fn create_label<'b>(
        &self,
        label: Label<(&'b str, Range<usize>)>,
    ) -> Label<(&'b str, Range<usize>)> {
        match self {
            Self::InvalidToken => label
                .with_message("Unable to parse this word")
                .with_color(Color::Red),
            Self::UnexpectedToken { expected, found } => {
                let label_msg = if found.is_none() {
                    format!("Expected {}, but file ended", expected)
                } else {
                    format!("Expected {}", expected)
                };
                label.with_message(label_msg).with_color(Color::Red)
            }
            Self::UnknownField { ident } => label
                .with_message(format!("Field `{}` is unknown", ident))
                .with_color(Color::Red),
            Self::InvalidAssignment => label
                .with_message("Cannot assign to this")
                .with_color(Color::Red),
            Self::IncompatibleType {
                expected_types: expected_type,
                found_type,
            } => label
                .with_message(format!(
                    "Type {} expected, `{}` found",
                    format_iter(expected_type),
                    found_type
                ))
                .with_color(Color::Red),
            Self::OutOfBounds { bounds } => label
                .with_message(format!(
                    "Only accepts value between {} and {}",
                    bounds.start(),
                    bounds.end()
                ))
                .with_color(Color::Red),
            Self::UnknownType { type_name } => label
                .with_message(format!("Type name `{}` is unknown", type_name))
                .with_color(Color::Red),
            Self::FieldAlreadyDeclared => label
                .with_message("This field is already declared")
                .with_color(Color::Red),
            Self::ElementAlreadyDeclared => label
                .with_message("This element is already declared")
                .with_color(Color::Red),
            Self::StringInLogic => label
                .with_message("Cannot use string in logic")
                .with_color(Color::Red),
            Self::FieldAccessOnly => label
                .with_message("Use with a field access by a dot")
                .with_color(Color::Red),
            Self::OutputsInExpression => label
                .with_message("Keyword `outputs` is only valid for assignment target")
                .with_color(Color::Red),
            Self::NodeDoesNotExist {
                component_str,
                index,
            } => label
                .with_message(format!(
                    "{} th output node does not exist in component {}",
                    index, component_str
                ))
                .with_color(Color::Red),
            Self::IncompatibleNodeType {
                expected_type,
                found_type,
            } => label
                .with_message(format!(
                    "Type `{}` expected, `{}` found",
                    expected_type, found_type
                ))
                .with_color(Color::Red),
        }
    }
}

fn format_iter<T, I>(iter: I) -> String
where
    T: std::fmt::Display,
    I: IntoIterator<Item = T>,
{
    let items: Vec<String> = iter.into_iter().map(|x| format!("`{x}`")).collect();
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        _ => format!(
            "{} or {}",
            items[..items.len() - 1].join(", "),
            items.last().unwrap()
        ),
    }
}
