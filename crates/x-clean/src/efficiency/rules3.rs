use super::{Detector, Spec};

pub(super) const SPECS: &[Spec] = &[
    Spec {
        identifier: "efficiency-extra-format-only-value",
        pattern: Detector::Ast,
        title: "Formatting one value",
        description: "Formatting contains only one Display value.",
        positive: "fn f() { let text = format!(\"{}\", value); }",
        negative: "fn f() { let text = value.to_string(); }",
    },
    Spec {
        identifier: "efficiency-extra-format-literal-only",
        pattern: Detector::Ast,
        title: "Formatting a literal",
        description: "Formatting contains no substitutions.",
        positive: "fn f() { let text = format!(\"hello\"); }",
        negative: "fn f() { let text = \"hello\"; }",
    },
    Spec {
        identifier: "efficiency-extra-string-literal-owned",
        pattern: Detector::Ast,
        title: "Owned string literal",
        description: "A string literal is converted to an owned String.",
        positive: "fn f() { let text = \"hello\".to_string(); }",
        negative: "fn f() { let text = \"hello\"; }",
    },
    Spec {
        identifier: "efficiency-extra-metadata-then-open",
        pattern: Detector::Tokens("(?:fs::metadata\\((\\w+)\\)[^;]*;)[^{}]*?File::open\\(\\1\\)"),
        title: "Metadata lookup before opening",
        description: "File metadata is queried before opening the file.",
        positive: "fn f() { std::fs::metadata(path); File::open(path); }",
        negative: "fn f() { std::fs::metadata(path); File::open(other); }",
    },
    Spec {
        identifier: "efficiency-extra-read-entire-for-lines",
        pattern: Detector::Tokens("fs::read_to_string\\([^;]*\\)\\.lines\\("),
        title: "Reading a whole file for lines",
        description: "A file is read into memory before processing its lines.",
        positive: "fn f() { std::fs::read_to_string(path).unwrap().lines(); }",
        negative: "fn f() { reader.lines(); }",
    },
];
