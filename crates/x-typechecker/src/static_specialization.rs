mod substitution;
mod traversal;
use std::collections::{HashMap, HashSet};
use std::slice;

use crate::{
    TypeChecker,
    errors::TypeError,
    specialization_key::{SpecializationKey, StaticValue},
};
use substitution::*;
use traversal::*;
use x_ast::DeclarationKey;
use x_ast::Parameter;
use x_ast::SourceError;
use x_ast::SourceSpan;
use x_ast::{Expression, ExpressionKind, Pattern, Statement, StringPart};
use x_diagnostics::frontend_messages::FrontendMessage;

const MAX_STATIC_SPECIALIZATIONS: usize = 4096;

struct StaticTemplate {
    statement: Statement,
    parameters: Vec<Parameter>,
    origin: Option<SourceSpan>,
}

struct StaticSpecialiser {
    templates: HashMap<String, StaticTemplate>,
    generated_names: HashMap<SpecializationKey, String>,
    occupied_names: HashSet<String>,
    generated: Vec<Statement>,
    generated_origins: HashMap<DeclarationKey, SourceSpan>,
}

impl TypeChecker {
    pub(crate) fn specialise_static_functions(
        &mut self,
        statements: Vec<Statement>,
    ) -> Result<Vec<Statement>, TypeError> {
        let templates = statements
            .iter()
            .filter_map(|statement| {
                let Statement::Function {
                    name,
                    parameters,
                    is_static,
                    ..
                } = statement
                else {
                    return None;
                };

                (*is_static || parameters.iter().any(|parameter| parameter.is_static)).then(|| {
                    (
                        name.clone(),
                        StaticTemplate {
                            statement: statement.clone(),
                            parameters: parameters.clone(),
                            origin: statement
                                .declaration_key()
                                .and_then(|key| self.declaration_sources.get(&key).copied()),
                        },
                    )
                })
            })
            .collect::<HashMap<_, _>>();

        if templates.is_empty() {
            return Ok(statements);
        }

        // LLVM functions and globals share a symbol namespace.
        let occupied_names = statements
            .iter()
            .filter_map(|statement| match statement {
                Statement::Function { name, .. }
                | Statement::ExternFunctionDeclaration { name, .. }
                | Statement::Constant { name, .. }
                | Statement::VariableDeclaration {
                    pattern: Pattern::Identifier(name),
                    ..
                } => Some(name.clone()),
                _ => None,
            })
            .collect();

        let mut output = statements
            .into_iter()
            .filter(|statement| {
                !matches!(statement, Statement::Function { name, .. } if templates.contains_key(name))
            })
            .collect::<Vec<_>>();
        let mut specialiser = StaticSpecialiser {
            templates,
            generated_names: HashMap::new(),
            occupied_names,
            generated: Vec::new(),
            generated_origins: HashMap::new(),
        };

        for statement in &mut output {
            specialiser.rewrite_statement(statement)?;
        }

        let mut index = 0;

        while let Some(mut generated) = specialiser.generated.get(index).cloned() {
            specialiser.rewrite_statement(&mut generated)?;
            let slot = specialiser.generated.get_mut(index).ok_or_else(|| {
                TypeError::invalid(FrontendMessage::ChangedStaticSpecialisationQueue)
            })?;
            *slot = generated;
            index = index.saturating_add(1);
        }

        self.declaration_sources
            .extend(specialiser.generated_origins);
        output.extend(specialiser.generated);

        Ok(output)
    }
}

impl StaticSpecialiser {
    fn rewrite_statement(&mut self, statement: &mut Statement) -> Result<(), TypeError> {
        visit_statement_expressions_mut(statement, DeclarationBodies::Enter, &mut |expression| {
            self.rewrite_expression(expression)
        })
    }

    fn rewrite_expression(&mut self, expression: &mut Expression) -> Result<(), TypeError> {
        self.rewrite_expression_inner(expression)
            .map_err(|error| error.at_source(expression.span))
    }

    fn rewrite_expression_inner(&mut self, expression: &mut Expression) -> Result<(), TypeError> {
        walk_expression_children_mut(expression, &mut |child| self.rewrite_expression(child))?;
        let ExpressionKind::FunctionCall {
            target: name,
            arguments: current_arguments,
        } = &mut expression.kind
        else {
            return Ok(());
        };

        let resolved_name = name.symbol_name().into_owned();
        let Some(template) = self.templates.get(&resolved_name) else {
            return Ok(());
        };

        if current_arguments.len() != template.parameters.len() {
            return Err(TypeError::ArityMismatch {
                expected: template.parameters.len(),
                got: current_arguments.len(),
                symbol_name: resolved_name.clone(),
            });
        }

        let mut substitutions = HashMap::new();
        let mut retained_arguments = Vec::new();
        let mut key = SpecializationKey {
            function: resolved_name.clone(),
            arguments: Vec::new(),
        };

        for (current_parameter, argument) in
            template.parameters.iter().zip(current_arguments.iter())
        {
            if current_parameter.is_static {
                let value_key = static_literal_key(argument).ok_or_else(|| {
                    TypeError::inference(FrontendMessage::NonLiteralStaticArgument {
                        parameter: current_parameter.name.to_string(),
                        function: resolved_name.to_string(),
                    })
                })?;
                key.arguments
                    .push((current_parameter.name.clone(), value_key));
                substitutions.insert(current_parameter.name.clone(), argument.clone());
            } else {
                retained_arguments.push(argument.clone());
            }
        }

        let specialised_name = if let Some(existing) = self.generated_names.get(&key) {
            existing.clone()
        } else {
            if self.generated.len() >= MAX_STATIC_SPECIALIZATIONS {
                return Err(TypeError::ResourceLimit {
                    resource: crate::errors::TypeResource::StaticSpecializations,
                    maximum: MAX_STATIC_SPECIALIZATIONS,
                });
            }

            let base_name = key.symbol();
            let mut specialised_name = base_name.clone();
            let mut disambiguator: usize = 0;

            while !self.occupied_names.insert(specialised_name.clone()) {
                disambiguator = disambiguator.checked_add(1).ok_or_else(|| {
                    TypeError::invalid(FrontendMessage::SpecialisationSymbolSpaceExhausted)
                })?;
                specialised_name = format!("{base_name}__{disambiguator}");
            }

            let mut specialised = template.statement.clone();
            let Statement::Function {
                name: clone_name,
                parameters,
                body,
                is_static,
                ..
            } = &mut specialised
            else {
                return Err(TypeError::inference(
                    FrontendMessage::NonFunctionStaticTemplate,
                ));
            };

            *clone_name = specialised_name.clone();
            parameters.retain(|current_parameter| !current_parameter.is_static);
            for current_parameter in parameters.iter_mut() {
                current_parameter.is_static = false;
            }

            if let Some(body) = body {
                ensure_static_substitution_is_unambiguous(body, &substitutions)?;
                substitute_statements(body, &substitutions)?;
            }
            *is_static = true;
            self.generated_names
                .insert(key.clone(), specialised_name.clone());
            if let (Some(origin), Some(key)) = (template.origin, specialised.declaration_key()) {
                self.generated_origins.insert(key, origin);
            }

            self.generated.push(specialised);

            specialised_name
        };

        name.resolve_to(specialised_name);
        *current_arguments = retained_arguments;

        Ok(())
    }
}
