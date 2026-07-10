use pest::Parser;

use crate::baseline::strip_semantic_features_in_program;
use crate::diagnostics::format_syntax_error;
use crate::script_entry::normalise_script_entrypoint;
use crate::{ParseOptions, ParsedModule, Rule, XParser, cst_validation, program, scalar};
use x_diagnostics::{CompilationFailure, parser_messages};

#[derive(Clone, Copy)]
pub(crate) enum EntryPoint {
    Script,
    Module,
}

pub(super) fn parse(
    input: &str,
    options: ParseOptions,
    entrypoint: EntryPoint,
) -> Result<ParsedModule, CompilationFailure> {
    let parsed = if let Some(parsed) = scalar::try_parse(input) {
        parsed
    } else {
        let pairs = XParser::parse(Rule::program, input).map_err(format_syntax_error)?;
        let program_pair = pairs.peek().ok_or_else(parser_messages::empty_input)?;
        let validated = cst_validation::validate_program_cst(&program_pair)
            .map_err(|error| error.diagnostic().into_failure(&error))?;

        program::lower_program(validated)?
    };

    let ParsedModule {
        mut program,
        item_spans,
        submodules,
    } = parsed;

    if options.disable_semantic_keywords {
        strip_semantic_features_in_program(&mut program);
    }

    if matches!(entrypoint, EntryPoint::Script) {
        normalise_script_entrypoint(&mut program);
    }

    program
        .validate_ast()
        .map_err(parser_messages::ast_validation)?;

    Ok(ParsedModule {
        program,
        item_spans,
        submodules,
    })
}
