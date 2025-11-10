use super::{NodeType, OptionalLink};

use std::borrow::Cow;

#[expect(dead_code)]
#[derive(strum::Display, Debug)]
#[repr(u8)]
pub enum Component {
    Add {
        input_a: OptionalLink,
        input_b: OptionalLink,
    },
    Subtract {
        input_a: OptionalLink,
        input_b: OptionalLink,
    },
    Multiply {
        input_a: OptionalLink,
        input_b: OptionalLink,
    },
    Divide {
        input_a: OptionalLink,
        input_b: OptionalLink,
    },
    #[strum(to_string = "f(x, y, z)")]
    Function3 {
        input_x: OptionalLink,
        input_y: OptionalLink,
        input_z: OptionalLink,
        function: String,
    },
    Clamp {
        input: OptionalLink,
        min: f32,
        max: f32,
    },
    Abs {
        input: OptionalLink,
    },
    #[strum(to_string = "Constant Number")]
    ConstantNumber {
        value: f32,
    },
    Delta {
        input: OptionalLink,
    },
    #[strum(to_string = "f(x, y, z, w, a, b, c, d)")]
    Function8 {
        input_x: OptionalLink,
        input_y: OptionalLink,
        input_z: OptionalLink,
        input_w: OptionalLink,
        input_a: OptionalLink,
        input_b: OptionalLink,
        input_c: OptionalLink,
        input_d: OptionalLink,
        function: String,
    },
    #[strum(to_string = "Modulo (fmod)")]
    Modulo {
        input_a: OptionalLink,
        input_b: OptionalLink,
    },
    Equal {
        input_a: OptionalLink,
        input_b: OptionalLink,
        epsilon: f32,
    },
    #[strum(to_string = "f(x)")]
    Function1 {
        input_x: OptionalLink,
        function: String,
    },
}

#[expect(dead_code)]
impl Component {
    pub fn component_type(&self) -> u8 {
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

    pub fn width(&self) -> u8 {
        4
    }

    pub fn height(&self) -> u8 {
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

    pub fn input_links(&self) -> Vec<&OptionalLink> {
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
            Self::Clamp { input, .. } | Self::Abs { input } | Self::Delta { input } => vec![input],
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

    pub fn inputs(&self) -> Cow<'static, [ComponentNode<'static>]> {
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

    pub fn outputs(&self) -> Cow<'static, [ComponentNode<'static>]> {
        match self {
            Self::Add { .. } => {
                Cow::Borrowed(&[ComponentNode(Cow::Borrowed("A + B"), NodeType::Number)])
            }
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
}

#[expect(dead_code)]
#[derive(Clone, Debug)]
pub struct ComponentNode<'a>(Cow<'a, str>, NodeType);
