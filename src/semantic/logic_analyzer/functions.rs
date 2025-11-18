use super::{LogicAnalyzer, Span};
use crate::{
    compile_error::CompileErrorType,
    microcontroller::{ArithmeticComponent, Component, Link},
    syntax::{Expr, Spanned},
};

impl<'f, 'e> LogicAnalyzer<'f, 'e> {
    pub(super) fn function_call(
        &mut self,
        func_type: &str,
        props: &Option<Spanned<Vec<Spanned<Expr>>>>,
        args: &Spanned<Vec<Spanned<Expr>>>,
        span: &Span,
    ) -> Option<Result<Link, CompileErrorType>> {
        let (component, index) = match func_type {
            "clamp" => {
                let (input,) = self.args_tuple1(args, "(input)")?;
                let (min, max) = self.props_tuple2(props, "{min, max}", span)?;
                (
                    Component::Arithmetic(ArithmeticComponent::Clamp {
                        input,
                        min: self.evaluate_expr(min)?,
                        max: self.evaluate_expr(max)?,
                    }),
                    0,
                )
            }
            "abs" => {
                let (input,) = self.args_tuple1(args, "(input)")?;
                (Component::Arithmetic(ArithmeticComponent::Abs { input }), 0)
            }
            _ => todo!(),
        };
        Some(self.add_component(component, index))
    }

    fn result_to_option<T>(&mut self, val: Result<T, CompileErrorType>, span: &Span) -> Option<T> {
        match val {
            Ok(v) => Some(v),
            Err(err) => {
                self.push_error(span.clone(), err);
                None
            }
        }
    }

    #[expect(dead_code)]
    fn props_tuple1<'a>(
        &mut self,
        props: &'a Option<Spanned<Vec<Spanned<Expr>>>>,
        expect_str: &'static str,
        span: &Span,
    ) -> Option<(&'a Spanned<Expr>,)> {
        match props {
            Some(props) => self.result_to_option(to_tuple1(&props.inner, expect_str), &props.span),
            None => {
                self.result_to_option(Err(CompileErrorType::PropertyRequired { expect_str }), span)
            }
        }
    }

    fn props_tuple2<'a>(
        &mut self,
        props: &'a Option<Spanned<Vec<Spanned<Expr>>>>,
        expect_str: &'static str,
        span: &Span,
    ) -> Option<(&'a Spanned<Expr>, &'a Spanned<Expr>)> {
        match props {
            Some(props) => self.result_to_option(to_tuple2(&props.inner, expect_str), &props.span),
            None => {
                self.result_to_option(Err(CompileErrorType::PropertyRequired { expect_str }), span)
            }
        }
    }

    fn args_tuple1<T1>(
        &mut self,
        args: &Spanned<Vec<Spanned<Expr>>>,
        expect_str: &'static str,
    ) -> Option<(T1,)>
    where
        T1: TryFrom<Link>,
        <T1 as TryFrom<Link>>::Error: Into<CompileErrorType>,
    {
        if let Some((a,)) = self.result_to_option(to_tuple1(&args.inner, expect_str), &args.span) {
            let a = self.expr_to_typed_link(a)?;
            Some((a,))
        } else {
            None
        }
    }

    #[expect(dead_code)]
    fn args_tuple2<T1, T2>(
        &mut self,
        args: &Spanned<Vec<Spanned<Expr>>>,
        expect_str: &'static str,
    ) -> Option<(T1, T2)>
    where
        T1: TryFrom<Link>,
        <T1 as TryFrom<Link>>::Error: Into<CompileErrorType>,
        T2: TryFrom<Link>,
        <T2 as TryFrom<Link>>::Error: Into<CompileErrorType>,
    {
        if let Some((a, b)) = self.result_to_option(to_tuple2(args, expect_str), &args.span) {
            let a = self.expr_to_typed_link(a)?;
            let b = self.expr_to_typed_link(b)?;
            Some((a, b))
        } else {
            None
        }
    }
}

fn to_tuple1<'a, T>(arr: &'a [T], expect_str: &'static str) -> Result<(&'a T,), CompileErrorType> {
    match arr {
        [a] => Ok((a,)),
        _ => Err(CompileErrorType::LengthMismatch {
            expect_str,
            found_len: arr.len(),
        }),
    }
}

fn to_tuple2<'a, T>(
    arr: &'a [T],
    expect_str: &'static str,
) -> Result<(&'a T, &'a T), CompileErrorType> {
    match arr {
        [a, b] => Ok((a, b)),
        _ => Err(CompileErrorType::LengthMismatch {
            expect_str,
            found_len: arr.len(),
        }),
    }
}
