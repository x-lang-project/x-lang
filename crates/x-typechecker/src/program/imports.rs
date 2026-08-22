use crate::{TypeChecker, errors::TypeError, qualify_module};
use x_ast::{Program, Statement};
use x_diagnostics::frontend_messages;
use x_std::StdLibraryCatalog;

impl TypeChecker {
    pub fn collect_all_statements(
        &mut self,
        statements: Vec<Statement>,
    ) -> Result<Vec<Statement>, TypeError> {
        self.expand_imports(statements, &StdLibraryCatalog::new())
    }

    fn expand_imports(
        &mut self,
        statements: Vec<Statement>,
        catalog: &StdLibraryCatalog,
    ) -> Result<Vec<Statement>, TypeError> {
        let mut expanded = Vec::new();

        for statement in statements {
            match statement {
                Statement::Block { statements } => {
                    expanded.extend(self.expand_imports(statements, catalog)?);
                }
                Statement::Import {
                    ref module,
                    ref item,
                } if module == "std" || module.starts_with("std::") => {
                    let path = format!("{module}::{item}");
                    let namespace = if module == "std" { item } else { &path };

                    if self.processed_files.insert(path.clone()) {
                        let mut imported = parse_catalog_module(catalog, &path)?;
                        qualify_module(&mut imported, namespace);
                        for declaration in &mut imported.statements {
                            if let Statement::Function { name, .. } = declaration {
                                self.symbols.qualified_imports.insert(name.clone());
                            }
                        }

                        expanded.extend(self.expand_imports(imported.statements, catalog)?);
                    }

                    expanded.push(statement);
                }
                _ => expanded.push(statement),
            }
        }

        Ok(expanded)
    }
}

fn parse_catalog_module(catalog: &StdLibraryCatalog, path: &str) -> Result<Program, TypeError> {
    let source = catalog.get_module(path).ok_or_else(|| {
        TypeError::Frontend(frontend_messages::missing_standard_library_module(
            path,
            &catalog.module_names().collect::<Vec<_>>(),
        ))
    })?;

    x_parser::parse_module_with_options(source, x_parser::ParseOptions::default()).map_err(
        |error| {
            TypeError::Frontend(frontend_messages::standard_library_parse_failure(
                path, source, error,
            ))
        },
    )
}
