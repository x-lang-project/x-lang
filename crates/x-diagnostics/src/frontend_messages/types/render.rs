use super::*;

impl TypeDiagnostic {
    pub(super) fn type_mismatch_diagnostic(
        got_span: Span,
        expected_span: Span,
        expected: String,
        got: String,
    ) -> Diagnostic {
        Error::new("type mismatch")
            .with_code("E0001")
            .with_label(Label::primary(got_span, format!("found `{got}`")))
            .with_label(Label::secondary(
                expected_span,
                format!("expected `{expected}`"),
            ))
            .with_fact(
                DiagnosticFact::new(
                    FactKind::Required,
                    format!("this position requires `{expected}`"),
                )
                .at(expected_span),
            )
            .with_fact(
                DiagnosticFact::new(
                    FactKind::Inferred,
                    format!("this expression was inferred as `{got}`"),
                )
                .at(got_span),
            )
            .with_fix(DiagnosticFix::high(format!(
                "change this expression so it produces `{expected}`"
            )))
            .with_fix(DiagnosticFix::medium(format!(
                "change the required type to `{got}` if that was the intended type"
            )))
            .with_fix(DiagnosticFix::low(
                "add an explicit cast or conversion if the conversion is intentional",
            ))
            .with_help("the expression and the required type need to agree")
    }

    pub(super) fn arity_mismatch_diagnostic(
        expected: usize,
        got: usize,
        symbol_name: String,
    ) -> Diagnostic {
        Error::new("arity mismatch")
            .with_code("E0003")
            .with_note(format!(
                "`{symbol_name}` expects {expected} argument(s), but {got} were supplied"
            ))
            .with_fact(DiagnosticFact::new(
                FactKind::Required,
                format!("`{symbol_name}` is declared with {expected} argument(s)"),
            ))
            .with_fact(DiagnosticFact::new(
                FactKind::Found,
                format!("this call supplies {got} argument(s)"),
            ))
            .with_fix(DiagnosticFix::high(format!(
                "pass exactly {expected} argument(s) to `{symbol_name}`"
            )))
            .with_fix(DiagnosticFix::medium(
                "call a different function or overload whose parameters match these arguments",
            ))
            .with_fix(DiagnosticFix::low(format!(
                "change `{symbol_name}`'s signature if this call is the intended API"
            )))
            .with_help("function calls must match the resolved function signature")
    }

    pub(super) fn unresolved_property_diagnostic(
        field: String,
        value_type: TypeName,
    ) -> Diagnostic {
        Error::new("unknown field or method")
            .with_code("E0004")
            .with_note(format!("`{field}` does not exist on `{value_type:?}`"))
            .with_fact(DiagnosticFact::new(
                FactKind::Inferred,
                format!("the receiver type is `{value_type:?}`"),
            ))
            .with_fact(DiagnosticFact::new(
                FactKind::Found,
                format!("requested member `{field}`"),
            ))
            .with_fix(DiagnosticFix::high(format!(
                "correct `{field}` to a field or method that exists on `{value_type:?}`"
            )))
            .with_fix(DiagnosticFix::medium(
                "change the receiver expression so it has the type you meant",
            ))
            .with_fix(DiagnosticFix::low(
                "add this field or method to the type if it belongs there",
            ))
            .with_help("the member name must exist on the receiver type")
    }

    pub(super) fn lookup_issue_diagnostic(issue: super::super::FrontendMessage) -> Diagnostic {
        use super::super::FrontendMessage;

        let name = match issue {
            FrontendMessage::UnknownStruct { name } => name,
            FrontendMessage::UnknownStructNamed { struct_name } => struct_name,
            other => other.to_string(),
        };

        Self::unknown_identifier_diagnostic(name)
    }

    pub(super) fn unknown_identifier_diagnostic(name: String) -> Diagnostic {
        let clean_name = &name;

        Error::new(format!("cannot find value `{clean_name}` in this scope"))
        .with_code("E0006")
        .with_note(format!("no local, function, import, or global named `{clean_name}` is visible here"))
        .with_fact(DiagnosticFact::new(
            FactKind::Found,
            format!("lookup failed here: no declaration named `{clean_name}` is visible"),
        ))
        .with_fact(DiagnosticFact::new(
            FactKind::Note,
            "name lookup searched local variables, function parameters, functions, imports, and module globals",
        ))
        .with_fix(DiagnosticFix::high(format!(
            "declare `{clean_name}` before using it"
        )))
        .with_fix(DiagnosticFix::medium(
            "import the module or qualify the name with its module path",
        ))
        .with_fix(DiagnosticFix::low("replace it with a similarly named visible value"))
            .with_help("a name must be declared, imported, or defined as a global before it can be used")
    }

    pub(super) fn immutable_parameter_assignment_diagnostic(name: String) -> Diagnostic {
        Error::new("immutable parameter assignment")
            .with_code("E0008")
            .with_note(format!(
                "function parameter `{name}` is immutable by default"
            ))
            .with_fact(DiagnosticFact::new(
                FactKind::Required,
                "parameters must be declared `mut` before they can be reassigned",
            ))
            .with_fact(DiagnosticFact::new(
                FactKind::Found,
                format!("assignment targets immutable parameter `{name}`"),
            ))
            .with_fix(DiagnosticFix::high(format!(
                "change the parameter to `mut {name}` if reassignment is intentional"
            )))
            .with_fix(DiagnosticFix::medium(
                "introduce a mutable local copy and assign to that instead",
            ))
            .with_help("function parameters are immutable unless marked `mut`")
    }

    pub(super) fn resource_limit_diagnostic(resource: &'static str, maximum: usize) -> Diagnostic {
        Error::new("compiler resource limit exceeded")
            .with_code("E0011")
            .with_note(format!(
                "{resource} exceeds the supported maximum of {maximum}"
            ))
            .with_fact(DiagnosticFact::new(
                FactKind::Required,
                format!("{resource} must not exceed {maximum}"),
            ))
            .with_fix(DiagnosticFix::high(
                "split the expression into named intermediate values",
            ))
            .with_help("reduce the nesting or break a recursive type-alias cycle")
    }

    pub(super) fn constant_index_output_of_bounds_diagnostic(
        index: i64,
        length: usize,
    ) -> Diagnostic {
        Error::new("constant array index out of bounds")
            .with_code("E0014")
            .with_note(format!(
                "index {index} cannot select an element of a fixed-size array of length {length}"
            ))
            .with_fact(DiagnosticFact::new(
                FactKind::Required,
                format!("the index must be in the half-open interval 0..{length}"),
            ))
            .with_fix(DiagnosticFix::high(
                "use a constant index that is within the declared array length",
            ))
            .with_help("constant out-of-bounds projections are rejected before MIR lowering")
    }
}

mod effects;
