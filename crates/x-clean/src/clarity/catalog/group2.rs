pub(super) fn rules() -> Vec<crate::Rule> {
    vec![
        crate::Rule {
            identifier: r####"clarity-extra-trailing-underscore"####,
            title: r####"Trailing underscore name"####,
            description: r####"Trailing underscore name."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){let owner_=1;}"####,
            negative: r####"fn demo(){let owner=1;}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-uppercase-local"####,
            title: r####"Uppercase local binding"####,
            description: r####"Uppercase local binding."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){let OWNER=1;}"####,
            negative: r####"fn demo(){let owner=1;}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-long-generic-name"####,
            title: r####"Long generic parameter"####,
            description: r####"Long generic parameter."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo<ExtremelyLongGenericParameterName>(){}"####,
            negative: r####"fn demo<Value>(){}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-digit-heavy-name"####,
            title: r####"Digit-heavy identifier"####,
            description: r####"Digit-heavy identifier."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){let source202610=1;}"####,
            negative: r####"fn demo(){let source202=1;}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-repeated-name-separator"####,
            title: r####"Repeated underscore separator"####,
            description: r####"Repeated underscore separator."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){let source__file=1;}"####,
            negative: r####"fn demo(){let source_file=1;}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-boolean-role-type"####,
            title: r####"Boolean role with explicit nonbool type"####,
            description: r####"Boolean role with explicit nonbool type."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){let is_ready:u32=1;}"####,
            negative: r####"fn demo(){let is_ready:bool=true;}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-mixed-case-local"####,
            title: r####"Mixed naming conventions in local"####,
            description: r####"Mixed naming conventions in local."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){let Source_file=1;}"####,
            negative: r####"fn demo(){let source_file=1;}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-long-name"####,
            title: r####"Extremely long identifier"####,
            description: r####"Extremely long identifier."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){let extraordinarily_long_local_binding_identifier_for_review=1;}"####,
            negative: r####"fn demo(){let descriptive_source_file=1;}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-nonascii-doc"####,
            title: r####"Non-ASCII documentation"####,
            description: r####"Non-ASCII documentation."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"/// café
fn demo(){}"####,
            negative: r####"/// cafe
fn demo(){}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-empty-doc"####,
            title: r####"Empty documentation attribute"####,
            description: r####"Empty documentation attribute."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"#[doc=""] fn demo(){}"####,
            negative: r####"#[doc="Executes the command."] fn demo(){}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-duplicate-doc-line"####,
            title: r####"Repeated documentation line"####,
            description: r####"Repeated documentation line."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"/// Same sentence.
/// Same sentence.
fn demo(){}"####,
            negative: r####"/// First sentence.
/// Second sentence.
fn demo(){}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-doc-trailing-space"####,
            title: r####"Trailing documentation whitespace"####,
            description: r####"Trailing documentation whitespace."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"#[doc="Description. "] fn demo(){}"####,
            negative: r####"#[doc="Description."] fn demo(){}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-doc-tab"####,
            title: r####"Tab in documentation"####,
            description: r####"Tab in documentation."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"#[doc="Description.	More."] fn demo(){}"####,
            negative: r####"#[doc="Description. More."] fn demo(){}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-doc-long-line"####,
            title: r####"Long documentation line"####,
            description: r####"Long documentation line."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"#[doc="Descriptive Descriptive Descriptive Descriptive Descriptive Descriptive Descriptive Descriptive Descriptive Descriptive Descriptive Descriptive "] fn demo(){}"####,
            negative: r####"#[doc="Descriptive short line."] fn demo(){}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-doc-unclosed-fence"####,
            title: r####"Unclosed documentation code fence"####,
            description: r####"Unclosed documentation code fence."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"/// ```rust
/// let value=1;
fn demo(){}"####,
            negative: r####"/// ```rust
/// let value=1;
/// ```
fn demo(){}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-doc-example-unwrap"####,
            title: r####"Unwrap in documentation example"####,
            description: r####"Unwrap in documentation example."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"/// ```rust
/// value.unwrap();
/// ```
fn demo(){}"####,
            negative: r####"/// ```rust
/// value.unwrap_or_default();
/// ```
fn demo(){}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-negated-equality"####,
            title: r####"Negated equality comparison"####,
            description: r####"Negated equality comparison."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){if !(first==second){run();}}"####,
            negative: r####"fn demo(){if first!=second{run();}}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-boolean-comparison"####,
            title: r####"Comparison to Boolean literal"####,
            description: r####"Comparison to Boolean literal."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){if ready==true{run();}}"####,
            negative: r####"fn demo(){if ready{run();}}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-literal-condition"####,
            title: r####"Literal Boolean condition"####,
            description: r####"Literal Boolean condition."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){if true{run();}}"####,
            negative: r####"fn demo(){if ready{run();}}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-double-negation"####,
            title: r####"Repeated unary negation"####,
            description: r####"Repeated unary negation."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){let ready=!!value;}"####,
            negative: r####"fn demo(){let ready=!value;}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-redundant-parentheses"####,
            title: r####"Nested parentheses"####,
            description: r####"Nested parentheses."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){let value=((input));}"####,
            negative: r####"fn demo(){let value=(input);}"####,
        },
        crate::Rule {
            identifier: r####"clarity-extra-empty-else"####,
            title: r####"Empty explicit else block"####,
            description: r####"Empty explicit else block."####,
            analysis: r####"Rust syntax and rustdoc"####,
            positive: r####"fn demo(){if ready{run();}else{}}"####,
            negative: r####"fn demo(){if ready{run();}else{stop();}}"####,
        },
    ]
}
