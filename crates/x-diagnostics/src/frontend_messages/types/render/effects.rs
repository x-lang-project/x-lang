use super::*;

impl TypeDiagnostic {
    pub(in crate::frontend_messages::types) fn purity_violation_diagnostic(
        caller: String,
        callee: String,
        missing: String,
    ) -> Diagnostic {
        Error::new("purity constraint violation")
            .with_code("E0002")
            .with_note(format!(
                "`{caller}` calls `{callee}`, which performs observable operations: {missing}"
            ))
            .with_fact(DiagnosticFact::new(
                FactKind::Required,
                format!("`{callee}` performs observable operations: {missing}"),
            ))
            .with_fact(DiagnosticFact::new(
                FactKind::Found,
                format!("`{caller}` cannot perform those operations under its purity contract"),
            ))
            .with_fix(DiagnosticFix::high(format!(
                "replace `{callee}` with a pure alternative"
            )))
            .with_fix(DiagnosticFix::medium(
                "move the observable operation into a function without a `pure` contract",
            ))
            .with_fix(DiagnosticFix::low(format!(
                "remove `pure` from `{caller}` if observable operations are intended"
            )))
            .with_json_payload(
                JsonDiagnosticPayload::new("PurityViolation")
                    .caller(caller)
                    .callee(callee)
                    .repair_policies([
                        "replace_impure_call",
                        "move_observable_operation",
                        "remove_pure_contract",
                    ]),
            )
            .with_help(
                "`pure` functions must have inspectable bodies and call only pure operations",
            )
    }

    pub(in crate::frontend_messages::types) fn impure_operation_diagnostic(
        function: String,
        operation: String,
        category: String,
    ) -> Diagnostic {
        Error::new("purity constraint violation")
        .with_code("E0002")
        .with_note(format!(
            "`{operation}` is not allowed in pure function `{function}` (observable operation: `{category}`)"
        ))
        .with_fact(DiagnosticFact::new(
            FactKind::Required,
            format!("operation performs `{category}`"),
        ))
        .with_fact(DiagnosticFact::new(
            FactKind::Found,
            format!("`{function}` has a `pure` contract"),
        ))
        .with_fix(DiagnosticFix::high(format!(
            "move `{operation}` to a function without a `pure` contract"
        )))
        .with_help("`pure` forbids observable operations")
    }
}
