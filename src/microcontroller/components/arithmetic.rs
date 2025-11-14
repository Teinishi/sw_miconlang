use super::{ComponentData, LinkNode, NodeType, NumberLink, single_attr};
use crate::xml_schema::{ObjectValue, ObjectValueTag};

use std::collections::HashMap;

#[expect(dead_code)]
#[derive(strum::Display, Debug)]
#[repr(u8)]
pub enum ArithmeticComponent {
    Add {
        input_a: NumberLink,
        input_b: NumberLink,
    },
    Subtract {
        input_a: NumberLink,
        input_b: NumberLink,
    },
    Multiply {
        input_a: NumberLink,
        input_b: NumberLink,
    },
    Divide {
        input_a: NumberLink,
        input_b: NumberLink,
    },
    #[strum(to_string = "f(x, y, z)")]
    Function3 {
        input_x: NumberLink,
        input_y: NumberLink,
        input_z: NumberLink,
        function: String,
    },
    Clamp {
        input: NumberLink,
        min: f32,
        max: f32,
    },
    Abs {
        input: NumberLink,
    },
    #[strum(to_string = "Constant Number")]
    ConstantNumber {
        value: f32,
    },
    Delta {
        input: NumberLink,
    },
    #[strum(to_string = "f(x, y, z, w, a, b, c, d)")]
    Function8 {
        input_x: NumberLink,
        input_y: NumberLink,
        input_z: NumberLink,
        input_w: NumberLink,
        input_a: NumberLink,
        input_b: NumberLink,
        input_c: NumberLink,
        input_d: NumberLink,
        function: String,
    },
    #[strum(to_string = "Modulo (fmod)")]
    Modulo {
        input_a: NumberLink,
        input_b: NumberLink,
    },
    Equal {
        input_a: NumberLink,
        input_b: NumberLink,
        epsilon: f32,
    },
    #[strum(to_string = "f(x)")]
    Function1 {
        input_x: NumberLink,
        function: String,
    },
}

impl ComponentData for ArithmeticComponent {
    fn component_type(&self) -> u8 {
        match self {
            Self::Add { .. } => 6,
            Self::Subtract { .. } => 7,
            Self::Multiply { .. } => 8,
            Self::Divide { .. } => 9,
            Self::Function3 { .. } => 10,
            Self::Clamp { .. } => 11,
            Self::Abs { .. } => 14,
            Self::ConstantNumber { .. } => 15,
            Self::Delta { .. } => 35,
            Self::Function8 { .. } => 36,
            Self::Modulo { .. } => 38,
            Self::Equal { .. } => 42,
            Self::Function1 { .. } => 45,
        }
    }

    fn height(&self) -> u8 {
        match self {
            Self::Clamp { .. }
            | Self::Abs { .. }
            | Self::ConstantNumber { .. }
            | Self::Delta { .. }
            | Self::Function1 { .. } => 2,
            Self::Add { .. }
            | Self::Subtract { .. }
            | Self::Multiply { .. }
            | Self::Divide { .. }
            | Self::Modulo { .. }
            | Self::Equal { .. } => 3,
            Self::Function3 { .. } => 4,
            Self::Function8 { .. } => 9,
        }
    }

    fn input_links_node(&self) -> Vec<&Option<LinkNode>> {
        match self {
            Self::Add { input_a, input_b }
            | Self::Subtract { input_a, input_b }
            | Self::Multiply { input_a, input_b }
            | Self::Divide { input_a, input_b }
            | Self::Modulo { input_a, input_b }
            | Self::Equal {
                input_a, input_b, ..
            } => vec![input_a, input_b],
            Self::Function3 {
                input_x,
                input_y,
                input_z,
                ..
            } => vec![input_x, input_y, input_z],
            Self::Clamp { input, .. } | Self::Abs { input } | Self::Delta { input } => {
                vec![input]
            }
            Self::ConstantNumber { .. } => vec![],
            Self::Function8 {
                input_x,
                input_y,
                input_z,
                input_w,
                input_a,
                input_b,
                input_c,
                input_d,
                ..
            } => vec![
                input_x, input_y, input_z, input_w, input_a, input_b, input_c, input_d,
            ],
            Self::Function1 { input_x, .. } => vec![input_x],
        }
    }

    fn attrs(&self) -> Option<HashMap<String, String>> {
        match self {
            Self::Function1 { function, .. }
            | Self::Function3 { function, .. }
            | Self::Function8 { function, .. } => Some(single_attr("e", function.to_owned())),
            _ => None,
        }
    }

    fn value_list(&self) -> Option<Vec<(ObjectValueTag, ObjectValue)>> {
        match self {
            Self::Clamp { min, max, .. } => Some(vec![
                (ObjectValueTag::Min, ObjectValue::new(*min)),
                (ObjectValueTag::Max, ObjectValue::new(*max)),
            ]),
            Self::ConstantNumber { value } => {
                Some(vec![(ObjectValueTag::N, ObjectValue::new(*value))])
            }
            Self::Equal { epsilon, .. } => {
                Some(vec![(ObjectValueTag::E, ObjectValue::new(*epsilon))])
            }
            _ => None,
        }
    }

    fn output_type(&self, index: usize) -> Option<NodeType> {
        match self {
            Self::Add { .. }
            | Self::Subtract { .. }
            | Self::Multiply { .. }
            | Self::Function3 { .. }
            | Self::Clamp { .. }
            | Self::Abs { .. }
            | Self::ConstantNumber { .. }
            | Self::Delta { .. }
            | Self::Function8 { .. }
            | Self::Modulo { .. }
            | Self::Function1 { .. } => (index == 0).then_some(NodeType::Number),
            Self::Divide { .. } => match index {
                0 => Some(NodeType::Number),
                1 => Some(NodeType::Bool),
                _ => None,
            },
            Self::Equal { .. } => (index == 0).then_some(NodeType::Bool),
        }
    }

    /*
    fn inputs(&self) -> Cow<'static, [ComponentNode<'static>]> {
        match self {
            Self::Add { .. }
            | Self::Subtract { .. }
            | Self::Multiply { .. }
            | Self::Divide { .. }
            | Self::Modulo { .. }
            | Self::Equal { .. } => Cow::Borrowed(&[
                ComponentNode(Cow::Borrowed("A"), NodeType::Number),
                ComponentNode(Cow::Borrowed("B"), NodeType::Number),
            ]),
            Self::Function3 { .. } => Cow::Borrowed(&[
                ComponentNode(Cow::Borrowed("X"), NodeType::Number),
                ComponentNode(Cow::Borrowed("Y"), NodeType::Number),
                ComponentNode(Cow::Borrowed("Z"), NodeType::Number),
            ]),
            Self::Clamp { .. } | Self::Abs { .. } | Self::Delta { .. } => {
                Cow::Borrowed(&[ComponentNode(
                    Cow::Borrowed("Input Number"),
                    NodeType::Number,
                )])
            }
            Self::ConstantNumber { .. } => Cow::Borrowed(&[]),
            Self::Function8 { .. } => Cow::Borrowed(&[
                ComponentNode(Cow::Borrowed("X"), NodeType::Number),
                ComponentNode(Cow::Borrowed("Y"), NodeType::Number),
                ComponentNode(Cow::Borrowed("Z"), NodeType::Number),
                ComponentNode(Cow::Borrowed("W"), NodeType::Number),
                ComponentNode(Cow::Borrowed("A"), NodeType::Number),
                ComponentNode(Cow::Borrowed("B"), NodeType::Number),
                ComponentNode(Cow::Borrowed("C"), NodeType::Number),
                ComponentNode(Cow::Borrowed("D"), NodeType::Number),
            ]),
            Self::Function1 { .. } => {
                Cow::Borrowed(&[ComponentNode(Cow::Borrowed("X"), NodeType::Number)])
            }
        }
    }

    fn outputs(&self) -> Cow<'static, [ComponentNode<'static>]> {
        match self {
            Self::Add { .. } => {
                Cow::Borrowed(&[ComponentNode(Cow::Borrowed("A + B"), NodeType::Number)])
            }
            Self::Add { .. } => Cow::Owned(()),
            Self::Subtract { .. } => {
                Cow::Borrowed(&[ComponentNode(Cow::Borrowed("A - B"), NodeType::Number)])
            }
            Self::Multiply { .. } => {
                Cow::Borrowed(&[ComponentNode(Cow::Borrowed("A X B"), NodeType::Number)])
            }
            Self::Divide { .. } => Cow::Borrowed(&[
                ComponentNode(Cow::Borrowed("A / B"), NodeType::Number),
                ComponentNode(Cow::Borrowed("Divide by Zero"), NodeType::Bool),
            ]),
            Self::Function3 { .. } => {
                Cow::Borrowed(&[ComponentNode(Cow::Borrowed("F(X, Y, Z)"), NodeType::Number)])
            }
            Self::Clamp { .. } => Cow::Borrowed(&[ComponentNode(
                Cow::Borrowed("Clamped Input"),
                NodeType::Number,
            )]),
            Self::Abs { .. } => Cow::Borrowed(&[ComponentNode(
                Cow::Borrowed("Absolute Value"),
                NodeType::Number,
            )]),
            Self::ConstantNumber { .. } => Cow::Borrowed(&[ComponentNode(
                Cow::Borrowed("Constant Value"),
                NodeType::Number,
            )]),
            Self::Delta { .. } => Cow::Borrowed(&[ComponentNode(
                Cow::Borrowed("Delta of Input Value"),
                NodeType::Number,
            )]),
            Self::Function8 { .. } => Cow::Borrowed(&[ComponentNode(
                Cow::Borrowed("F(X, Y, Z, W, A, B, C, D)"),
                NodeType::Number,
            )]),
            Self::Modulo { .. } => {
                Cow::Borrowed(&[ComponentNode(Cow::Borrowed("A % B"), NodeType::Number)])
            }
            Self::Equal { .. } => {
                Cow::Borrowed(&[ComponentNode(Cow::Borrowed("A = B"), NodeType::Bool)])
            }
            Self::Function1 { .. } => {
                Cow::Borrowed(&[ComponentNode(Cow::Borrowed("F(X)"), NodeType::Number)])
            }
        }
    }
    */
}
