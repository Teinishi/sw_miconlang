mod compile_error_type;
pub use compile_error_type::CompileErrorType;

use ariadne::{Label, Report, ReportKind, Source};
use std::ops::Range;

#[derive(Debug)]
pub struct CompileError<'a> {
    filename: &'a str,
    span: Range<usize>,
    error_type: CompileErrorType,
}

impl<'a> CompileError<'a> {
    pub fn new(filename: &'a str, span: Range<usize>, error_type: CompileErrorType) -> Self {
        Self {
            filename,
            span,
            error_type,
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
