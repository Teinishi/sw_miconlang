mod compile_error_type;
pub use compile_error_type::CompileErrorType;

use crate::lexical::Token;
use ariadne::{Label, Report, ReportKind, Source};
use std::ops::Range;

#[derive(Debug)]
pub struct CompileError<'a, 'b> {
    filename: &'a str,
    span: Range<usize>,
    error_type: CompileErrorType<'b>,
}

impl<'a, 'b> CompileError<'a, 'b> {
    pub fn invalid_token(filename: &'a str, span: Range<usize>) -> Self {
        Self {
            filename,
            span,
            error_type: CompileErrorType::InvalidToken,
        }
    }

    pub fn unexpected_token(
        filename: &'a str,
        e: &'b chumsky::error::Rich<'_, Token, Range<usize>>,
    ) -> Self {
        Self {
            filename,
            span: e.span().clone(),
            error_type: CompileErrorType::unexpected_token(e),
        }
    }

    pub fn print(&self, cache: &Source<&str>) {
        Report::build(ReportKind::Error, (self.filename, self.span.clone()))
            .with_message(format!("Syntax Error: {}", self.error_type.name()))
            .with_label(
                self.error_type
                    .create_label(Label::new((self.filename, self.span.clone()))),
            )
            .finish()
            .eprint((self.filename, cache))
            .unwrap();
    }
}
