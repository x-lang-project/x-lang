use std::mem;
use std::slice;

use std::collections::HashSet;

use self::source::collect_source_namespace_names;
use crate::{TypeChecker, errors::TypeError};
use x_ast::Program;
use x_diagnostics::frontend_messages::FrontendIssue;

mod imports;
mod normalization;
mod source;

impl TypeChecker {
    pub fn check(&mut self, program: Program) -> Result<Program, TypeError> {
        self.check_in_place(program)?;

        Ok(self.program.clone())
    }

    pub fn check_and_take_program(&mut self, program: Program) -> Result<Program, TypeError> {
        self.check_in_place(program)?;

        Ok(mem::replace(
            &mut self.program,
            Program {
                statements: Vec::new(),
            },
        ))
    }

    fn check_in_place(&mut self, program: Program) -> Result<(), TypeError> {
        program.validate_ast().map_err(TypeError::AstBoundary)?;
        self.register_internal_helpers();

        // Qualified source declarations still need their bodies checked.
        let mut source_names = HashSet::new();

        collect_source_namespace_names(&program.statements, &mut source_names);
        let mut statements = self.collect_all_statements(program.statements)?;

        self.index_global_symbols(&statements)?;
        self.synthesise_default_methods(&mut statements)?;
        self.validate_trait_impls(&statements)?;

        self.check_source_statements(&mut statements, &source_names)?;
        self.normalize_checked_types(&mut statements)?;
        self.check_pending_monomorphs()?;
        statements.append(&mut self.pending_monomorphs);

        statements = self.specialise_static_functions(statements)?;
        self.normalize_checked_types(&mut statements)?;

        self.program.statements = statements;
        self.program
            .validate_ast()
            .map_err(TypeError::AstBoundary)?;

        Ok(())
    }

    fn check_pending_monomorphs(&mut self) -> Result<(), TypeError> {
        let mut monomorph_index = 0;

        while let Some(mut statement) = self.pending_monomorphs.get(monomorph_index).cloned() {
            self.check_statement(&mut statement)?;
            self.normalize_checked_types(slice::from_mut(&mut statement))?;
            let slot =
                self.pending_monomorphs
                    .get_mut(monomorph_index)
                    .ok_or(TypeError::Invariant(
                        FrontendIssue::MonomorphisationQueueChanged,
                    ))?;
            *slot = statement;
            monomorph_index = monomorph_index.saturating_add(1);
        }

        Ok(())
    }
}
