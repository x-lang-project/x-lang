use super::*;

impl BorrowDiagnostic {
    pub(super) fn undeclared_variable_diagnostic(name: String, span: Span) -> Diagnostic {
        Error::new("undeclared variable")
            .with_code("B0001")
            .with_label(Label::primary(
                span,
                format!("`{name}` is not declared here"),
            ))
            .with_fact(
                DiagnosticFact::new(
                    FactKind::Found,
                    format!("the name `{name}` has no visible binding"),
                )
                .at(span),
            )
            .with_fix(DiagnosticFix::high(format!(
                "declare `{name}` before this use"
            )))
            .with_fix(DiagnosticFix::medium(
                "move this use into the scope where the value is declared",
            ))
            .with_fix(DiagnosticFix::low(
                "check whether the name is misspelled or needs a module qualifier",
            ))
            .with_help("declare the variable before using it")
    }

    pub(super) fn use_after_move_diagnostic(name: String, span: Span) -> Diagnostic {
        Error::new("use of moved value")
            .with_code("B0002")
            .with_label(Label::primary(span, format!("`{name}` was already moved")))
            .with_fact(
                DiagnosticFact::new(
                    FactKind::Found,
                    format!("`{name}` no longer owns a value at this use"),
                )
                .at(span),
            )
            .with_fix(DiagnosticFix::high(format!(
                "borrow `{name}` instead of moving it earlier"
            )))
            .with_fix(DiagnosticFix::medium(format!(
                "clone or copy `{name}` before the move if duplication is intended"
            )))
            .with_fix(DiagnosticFix::low(
                "reorder the code so this is the final use after ownership moves",
            ))
            .with_help("ownership moved earlier, so this binding cannot be used here")
    }

    pub(super) fn use_before_initialise_diagnostic(name: String, span: Span) -> Diagnostic {
        Error::new("use of uninitialised value")
            .with_code("B0011")
            .with_label(Label::primary(
                span,
                format!("`{name}` is not definitely initialised"),
            ))
            .with_fact(
                DiagnosticFact::new(
                    FactKind::Found,
                    format!("`{name}` may not have a value on every path"),
                )
                .at(span),
            )
            .with_fix(DiagnosticFix::high(format!(
                "assign `{name}` before this read on every control-flow path"
            )))
            .with_fix(DiagnosticFix::medium(
                "give the variable an initialiser when it is declared",
            ))
            .with_fix(DiagnosticFix::low(
                "return early on paths where the value cannot be initialised",
            ))
            .with_help("assign a value before reading or borrowing it")
    }

    pub(super) fn move_while_borrowed_diagnostic(name: String, span: Span) -> Diagnostic {
        Error::new("cannot move borrowed value")
            .with_code("B0003")
            .with_label(Label::primary(span, format!("`{name}` is still borrowed")))
            .with_fact(
                DiagnosticFact::new(
                    FactKind::Found,
                    format!("`{name}` has at least one active borrow"),
                )
                .at(span),
            )
            .with_fix(DiagnosticFix::high(format!(
                "end the borrow of `{name}` before moving it"
            )))
            .with_fix(DiagnosticFix::medium(format!(
                "move a clone or copied value instead of `{name}`"
            )))
            .with_fix(DiagnosticFix::low(
                "rewrite the code so the owner outlives every borrow",
            ))
            .with_help("end the borrow before moving the value")
    }

    pub(super) fn mutable_borrow_violation_diagnostic(name: String, span: Span) -> Diagnostic {
        Error::new("conflicting mutable borrow")
            .with_code("B0004")
            .with_label(Label::primary(
                span,
                format!("`{name}` cannot be borrowed as mutable here"),
            ))
            .with_fact(
                DiagnosticFact::new(FactKind::Found, format!("`{name}` is already borrowed"))
                    .at(span),
            )
            .with_fix(DiagnosticFix::high(
                "end the existing borrow before taking a mutable borrow",
            ))
            .with_fix(DiagnosticFix::medium(
                "use an immutable borrow if mutation is not needed",
            ))
            .with_fix(DiagnosticFix::low(
                "split the value so independent fields are borrowed separately",
            ))
            .with_help("ensure there is only one active mutable borrow")
    }

    pub(super) fn immutable_borrow_violation_diagnostic(name: String, span: Span) -> Diagnostic {
        Error::new("conflicting immutable borrow")
            .with_code("B0005")
            .with_label(Label::primary(
                span,
                format!("`{name}` cannot be borrowed as immutable here"),
            ))
            .with_fact(
                DiagnosticFact::new(
                    FactKind::Found,
                    format!("`{name}` is already mutably borrowed"),
                )
                .at(span),
            )
            .with_fix(DiagnosticFix::high(
                "end the mutable borrow before taking an immutable borrow",
            ))
            .with_fix(DiagnosticFix::medium(
                "perform the read through the existing mutable reference",
            ))
            .with_fix(DiagnosticFix::low(
                "move the read before the mutable borrow starts",
            ))
            .with_help("end the mutable borrow before taking an immutable borrow")
    }

    pub(super) fn lifetime_too_short_diagnostic(target: String, span: Span) -> Diagnostic {
        Error::new("borrowed value does not live long enough")
            .with_code("B0006")
            .with_label(Label::primary(
                span,
                format!("reference to `{target}` outlives its scope"),
            ))
            .with_fact(
                DiagnosticFact::new(
                    FactKind::Found,
                    format!("the reference to `{target}` would escape its valid scope"),
                )
                .at(span),
            )
            .with_fix(DiagnosticFix::high(
                "return or store an owned value instead of a reference",
            ))
            .with_fix(DiagnosticFix::medium(format!(
                "move `{target}` to a scope that lives long enough"
            )))
            .with_fix(DiagnosticFix::low(
                "use an explicit lifetime contract for a returned reference",
            ))
            .with_help("return an owned value or move the referenced value to a longer-lived scope")
    }

    pub(super) fn non_l_value_reference_diagnostic(kind: DebugValue, span: Span) -> Diagnostic {
        Error::new("cannot borrow non-lvalue expression")
            .with_code("B0010")
            .with_label(Label::primary(
                span,
                format!("`{kind:?}` is not a stable storage location"),
            ))
            .with_help("borrow a named local, field, array slot, or dereferenceable pointer")
    }
}
