use std::collections::HashMap;
use std::path::{Path, PathBuf};

use x_ast::DeclarationKey;
use x_ast::SourceSpan;
use x_ast::{Expression, Program, Statement, Type, TypeVisitorMut};
use x_diagnostics::CompilationFailure;
use x_diagnostics::contract_messages;

#[derive(Default)]
pub struct SourceCatalog {
    files: Vec<(PathBuf, String)>,
    pub(crate) declarations: HashMap<DeclarationKey, SourceSpan>,
}

impl SourceCatalog {
    pub fn declaration_spans(&self) -> &HashMap<DeclarationKey, SourceSpan> {
        &self.declarations
    }

    pub fn source_for(&self, span: Option<SourceSpan>) -> Option<(&Path, &str)> {
        span.and_then(|span| span.file_identifier)
            .and_then(|identifier| self.get(identifier))
    }

    pub fn register(
        &mut self,
        program: &mut Program,
        path: impl AsRef<Path>,
        source: &str,
        item_spans: &[Option<SourceSpan>],
    ) -> Result<(), CompilationFailure> {
        if item_spans.len() != program.statements.len() {
            return Err(contract_messages::source_declaration_mismatch());
        }

        let identifier = u32::try_from(self.files.len())
            .map_err(|_| contract_messages::source_capacity_exceeded())?;
        self.files
            .push((path.as_ref().to_owned(), source.to_owned()));
        let mut marker = SourceMarker(identifier);

        for (index, statement) in program.statements.iter_mut().enumerate() {
            marker.visit_statement(statement);
            let origin = item_spans.get(index).copied().flatten().or_else(|| {
                if let Statement::ImplDeclaration(definition) = statement
                    && let Type::Custom(name) | Type::GenericInstance { name, .. } =
                        &definition.type_name
                {
                    return self
                        .declarations
                        .get(&DeclarationKey::Struct(name.clone()))
                        .copied();
                }

                None
            });

            if let (Some(key), Some(mut span)) = (statement.declaration_key(), origin) {
                span.file_identifier = Some(identifier);
                self.declarations.insert(key, span);
            }
        }

        Ok(())
    }

    pub(crate) fn get(&self, identifier: u32) -> Option<(&Path, &str)> {
        self.files
            .get(identifier as usize)
            .map(|(path, source)| (path.as_path(), source.as_str()))
    }
}

struct SourceMarker(u32);

impl TypeVisitorMut for SourceMarker {
    fn visit_type(&mut self, _: &mut Type) {}

    fn visit_expression(&mut self, expression: &mut Expression) {
        if let Some(span) = &mut expression.span {
            span.file_identifier = Some(self.0);
        }

        x_ast::walk_expression_types_mut(self, expression);
    }

    fn visit_statement(&mut self, statement: &mut Statement) {
        match statement {
            Statement::StructDeclaration(definition) => {
                if let Some(invariant) = &mut definition.invariant {
                    self.visit_expression(&mut invariant.predicate);
                }
            }
            Statement::TraitDeclaration(definition) => {
                for method in definition.methods.values_mut().flatten() {
                    self.visit_statement(method);
                }
            }
            Statement::ImplDeclaration(definition) => {
                for method in &mut definition.methods {
                    self.visit_statement(method);
                }
            }
            _ => x_ast::walk_statement_types_mut(self, statement),
        }
    }
}
