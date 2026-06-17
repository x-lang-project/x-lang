use super::{Detector, Spec};

pub(super) const SPECS: &[Spec] = &[
    Spec {
        identifier: "efficiency-extra-chars-count-byte-context",
        pattern: Detector::Tokens("\\.chars\\(\\)\\.count\\(\\)"),
        title: "Character counting",
        description: "chars().count() counts Unicode scalars, not bytes.",
        positive: "fn f() { text.chars().count(); }",
        negative: "fn f() { text.len(); }",
    },
    Spec {
        identifier: "efficiency-extra-string-byte-vector-copy",
        pattern: Detector::Tokens("\\.as_bytes\\(\\)\\.to_vec\\("),
        title: "Copying string bytes",
        description: "String bytes are copied into a vector.",
        positive: "fn f() { text.as_bytes().to_vec(); }",
        negative: "fn f() { text.as_bytes(); }",
    },
];
