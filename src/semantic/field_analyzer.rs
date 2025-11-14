use crate::{
    compile_error::{CompileError, CompileErrorType},
    syntax::{Assignment, Expr, Spanned},
};

use std::collections::HashSet;

#[derive(Debug)]
pub(super) struct FieldAnalyzer<'a> {
    known_field: HashSet<String>,
    filename: &'a str,
}

impl<'a> FieldAnalyzer<'a> {
    pub(super) fn new(filename: &'a str) -> Self {
        Self {
            known_field: HashSet::new(),
            filename,
        }
    }

    pub(super) fn assignment<F>(
        &mut self,
        assignment: &Spanned<Assignment>,
        callback: F,
    ) -> Result<(), CompileError<'a>>
    where
        F: FnOnce(&String, &Spanned<Expr>) -> Result<bool, CompileError<'a>>,
    {
        if let Expr::Ident(ident) = &assignment.target.inner {
            if self.known_field.contains(ident) {
                return Err(CompileError::new(
                    self.filename,
                    assignment.span.clone(),
                    CompileErrorType::FieldAlreadyDeclared,
                ));
            }
            if !callback(ident, &assignment.value)? {
                return Err(CompileError::new(
                    self.filename,
                    assignment.span.clone(),
                    CompileErrorType::UnknownField {
                        ident: ident.to_owned(),
                    },
                ));
            }
            self.known_field.insert(ident.to_owned());
        } else {
            return Err(CompileError::new(
                self.filename,
                assignment.target.span.clone(),
                CompileErrorType::InvalidAssignment,
            ));
        }

        Ok(())
    }
}
