use super::{Detector, Spec};

pub(super) const SPECS: &[Spec] = &[
    Spec {
        identifier: "buffered-write-flush-policy",
        pattern: Detector::Ast,
        title: "Buffered writer dropped without observable flush handling",
        description: "BufWriter construction and write-shaped call in a block without explicit flush",
        positive: "fn f() { let mut w = BufWriter::new(file); w.write_all(data); }",
        negative: "fn f() { let mut w = BufWriter::new(file); w.flush(); }",
    },
    Spec {
        identifier: "clone-before-move",
        pattern: Detector::Ast,
        title: "Clone before last use",
        description: "Local cloned at its final lexical occurrence; move legality is not proven",
        positive: "fn f() { let value = input; consume(value.clone()); }",
        negative: "fn f() { let value = input; consume(value.clone()); consume(value); }",
    },
    Spec {
        identifier: "collect-length-only",
        pattern: Detector::Ast,
        title: "Collection used only for length",
        description: "Explicit Vec collection is immediately queried for length",
        positive: "fn f() { values.collect::<Vec<_>>().len(); }",
        negative: "fn f() { values.count(); }",
    },
    Spec {
        identifier: "full-count-for-existence",
        pattern: Detector::Tokens("\\.count\\(\\)(?:>|!=|==)0"),
        title: "Full traversal for existence",
        description: "Iterator count compared to zero or collected len checked for emptiness",
        positive: "fn f() { if values.count() > 0 {} }",
        negative: "fn f() { if values.next().is_some() {} }",
    },
    Spec {
        identifier: "ignored-result-operation",
        pattern: Detector::Tokens(
            "(?:let_=|drop\\()[^;]*(?:\\.write(?:_all)?\\(|\\.flush\\(|\\.wait\\(|std::fs::\\w+\\()",
        ),
        title: "Fallible operation explicitly ignored",
        description: "let _ or drop used with I/O/process/synchronization Result",
        positive: "fn f() { let _ = file.write_all(data); }",
        negative: "fn f() -> Result<(), Error> { file.write_all(data)?; Ok(()) }",
    },
    Spec {
        identifier: "index-range-loop",
        pattern: Detector::Tokens("for(\\w+)in0\\.\\.(\\w+)\\.len\\(\\)\\{[^{}]*?\\2\\[\\1\\]"),
        title: "Range used solely for indexing",
        description: "for 0..collection.len where index only accesses that collection",
        positive: "fn f() { for index in 0..values.len() { consume(values[index]); } }",
        negative: "fn f() { for value in values { consume(value); } }",
    },
];
