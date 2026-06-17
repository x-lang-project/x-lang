use super::test_functions::{Function, functions};
use super::*;

pub(super) fn analyze(source: &Source, output: &mut Vec<Finding>) {
    let current_functions = functions(source);
    let self_cmp = regex::Regex::new(r"assert_eq!\((\w+),(\w+)\)").unwrap();

    for function in current_functions.iter().filter(|function| function.test) {
        let code = &function.code;
        let mut hit = |identifier: &str, message: &str| {
            source.emit(output, identifier, function.offset, message)
        };

        if !function.assertion && !has_attribute(&function.attributes, "should_panic") {
            hit(
                "structure-test-assertion-absence",
                "Test body contains no visible assertion/check or fallible return",
            );
        }

        if function.unchecked_branch {
            hit(
                "structure-test-branch-coverage-shape",
                "Test has a conditional check with no visible else verification; syntactic path hint",
            );
        }

        if function.body.contains("Command::new(\"")
            && !function.body.contains("which")
            && !function.body.contains("exists(")
        {
            hit(
                "structure-test-external-tool-assumption",
                "Test launches literal executable without a visible availability guard",
            );
        }

        if ["set_var(", "remove_var(", "set_current_dir("]
            .iter()
            .any(|text| code.contains(text))
        {
            hit(
                "structure-test-global-state-mutation",
                "Test mutates environment or current directory",
            );
        }

        if code.contains("sleep(") {
            hit(
                "structure-test-sleep-synchronization",
                "Test uses fixed sleep; review synchronisation",
            );
        }

        if code.contains("assert!(") && code.contains(".is_err()") {
            hit(
                "structure-weak-error-assertion",
                "Test asserts generic error status without visible error-kind verification",
            );
        }

        if code.contains("assert!(true)") || code.contains("assert!(false)") {
            hit(
                "structure-extra-test-constant-assertion",
                "Test assertion condition is a Boolean literal",
            );
        }

        if self_cmp
            .captures_iter(code)
            .any(|character| character[1] == character[2])
        {
            hit(
                "structure-extra-test-self-comparison-assertion",
                "Assertion compares the same identifier to itself",
            );
        }

        if code.contains("SystemTime::now(") || code.contains("Utc::now(") {
            hit(
                "structure-extra-test-runtime-clock",
                "Test reads wall-clock time; review reproducibility",
            );
        }

        if code.contains("process::exit(") {
            hit(
                "structure-extra-test-process-exit",
                "Test can terminate the process",
            );
        }

        for first_value in &function.attributes {
            if first_value.path().is_ident("ignore") {
                hit(
                    "structure-ignored-test-inventory",
                    "Ignored test: inspect reason and remaining coverage",
                );

                if matches!(first_value.meta, syn::Meta::Path(_)) {
                    hit(
                        "structure-extra-test-ignored-no-reason",
                        "Ignored test omits a reason string",
                    );
                }
            }

            if first_value.path().is_ident("should_panic")
                && !compact(first_value).contains("expected=")
            {
                hit(
                    "structure-extra-test-should-panic-no-message",
                    "Panic test does not constrain expected panic message",
                );
            }
        }
    }

    let tests: Vec<_> = current_functions
        .iter()
        .filter(|function| function.test)
        .collect();
    let path = regex::Regex::new(r#"(?:write|create|read|read_to_string)\("([^"]+)""#).unwrap();
    let mut resources: std::collections::HashMap<String, Vec<(&Function, bool)>> =
        std::collections::HashMap::new();

    for function in &tests {
        for character in path.captures_iter(&function.body) {
            let writing = character[0].starts_with("write") || character[0].starts_with("create");
            resources
                .entry(character[1].to_string())
                .or_default()
                .push((function, writing));
        }
    }

    for (name, uses) in resources {
        let writers: std::collections::HashSet<_> = uses
            .iter()
            .filter(|(_, writer)| *writer)
            .map(|(function, _)| &function.name)
            .collect();

        if writers.len() >= 2 {
            source.emit(
                output,
                "structure-test-fixed-filesystem-path",
                uses[0].0.offset,
                format!("Multiple tests write fixed path {name:?}"),
            );
        }

        if !writers.is_empty()
            && uses
                .iter()
                .any(|(function, writer)| !*writer && !writers.contains(&function.name))
        {
            source.emit(output,"structure-test-order-dependency-hint",uses[0].0.offset,format!("Different tests produce and consume shared path {name:?}; fixture setup may be external"));
        }
    }

    let normalize = regex::Regex::new(r#""[^"]*"|\b\d+\b"#).unwrap();

    for (index, function) in tests.iter().enumerate() {
        if function.body.matches(';').count() >= 2
            && tests[..index].iter().any(|pointer| {
                normalize.replace_all(&pointer.body, "LITERAL")
                    == normalize.replace_all(&function.body, "LITERAL")
            })
        {
            source.emit(
                output,
                "structure-test-table-opportunity",
                function.offset,
                "Tests repeat setup/action/assert syntax with literal variation",
            );
        }
    }

    for function in current_functions
        .iter()
        .filter(|function| !function.test && function.name.starts_with("test_"))
    {
        let called_in_test = tests
            .iter()
            .any(|type_value| type_value.body.contains(&format!("{}(", function.name)));
        let called_elsewhere = current_functions.iter().any(|pointer| {
            !pointer.test
                && pointer.name != function.name
                && pointer.body.contains(&format!("{}(", function.name))
        });

        if called_in_test && !called_elsewhere {
            source.emit(output,"structure-test-helper-placement",function.offset,"Helper named test_* is referenced only by visible tests; external/cfg uses unresolved");
        }
    }
}
