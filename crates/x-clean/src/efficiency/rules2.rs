use super::{Detector, Spec};

pub(super) const SPECS: &[Spec] = &[
    Spec {
        identifier: "unsafe-missing-contract",
        pattern: Detector::Ast,
        title: "Unsafe operation with no nearby safety rationale",
        description: "unsafe block/function/impl lacks structured SAFETY comment or contract doc",
        positive: "fn f() { unsafe { operation(); } }",
        negative: "fn f() { // SAFETY: pointer is valid\n unsafe { operation(); } }",
    },
    Spec {
        identifier: "zero-length-index",
        pattern: Detector::Ast,
        title: "Index access without visible empty guard",
        description: "Direct index-zero access requiring emptiness review",
        positive: "fn f() { consume(values[0]); }",
        negative: "fn f() { consume(values.get(0)); }",
    },
    Spec {
        identifier: "efficiency-extra-enumerate-discard-index",
        pattern: Detector::Tokens("\\.enumerate\\(\\)\\.map\\(\\|\\(_,?(\\w+)\\)\\|\\1\\)"),
        title: "Unused enumeration indices",
        description: "Enumeration indices are discarded.",
        positive: "fn f() { values.enumerate().map(|(_, item)| item); }",
        negative: "fn f() { values.enumerate().map(|(index,item)| (index,item)); }",
    },
    Spec {
        identifier: "efficiency-extra-reverse-sort",
        pattern: Detector::Tokens("\\.sort(?:_unstable)?\\(\\);\\w+\\.reverse\\("),
        title: "Reversing sorted results",
        description: "A sorted collection is reversed.",
        positive: "fn f() { values.sort(); values.reverse(); }",
        negative: "fn f() { values.sort_by(|a,b| b.cmp(a)); }",
    },
    Spec {
        identifier: "efficiency-extra-reverse-next",
        pattern: Detector::Tokens("\\.rev\\(\\)\\.next\\("),
        title: "Reversing before next",
        description: "An iterator is reversed before reading one element.",
        positive: "fn f() { values.rev().next(); }",
        negative: "fn f() { values.next_back(); }",
    },
];
