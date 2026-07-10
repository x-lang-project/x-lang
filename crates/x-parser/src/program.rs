use crate::cst_validation::ValidatedProgramCst;
use crate::derives::expand_builtin_derives;
use crate::{ParsedModule, Rule, source_span_from_pair, statement};
use x_ast::Program;
use x_diagnostics::{CompilationFailure, ParserDiagnostic, parser_messages};

pub(super) fn lower_program(
    validated: ValidatedProgramCst<'_>,
) -> Result<ParsedModule, CompilationFailure> {
    let pair = validated.into_pair();
    let mut statements = Vec::new();
    let mut item_spans = Vec::new();
    let mut submodules = Vec::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::statement => {
                let inner = pair
                    .clone()
                    .into_inner()
                    .next()
                    .ok_or_else(parser_messages::missing_statement_declaration)?;

                if inner.as_rule() == Rule::mod_declaration {
                    let name = inner
                        .into_inner()
                        .next()
                        .ok_or_else(parser_messages::missing_module_name)?;
                    submodules.push(name.as_str().to_owned());
                }

                item_spans.push(Some(source_span_from_pair(&pair)));
                statements
                    .push(statement::parse_statement(pair).map_err(|error| {
                        ParserDiagnostic::StatementLowering.into_failure(error)
                    })?);
            }

            Rule::EOI => break,
            unexpected => {
                return Err(parser_messages::unexpected_program_rule(unexpected));
            }
        }
    }

    expand_builtin_derives(&mut statements);

    item_spans.resize(statements.len(), None);

    Ok(ParsedModule {
        program: Program { statements },
        item_spans,
        submodules,
    })
}
