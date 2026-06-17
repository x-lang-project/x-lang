use crate::Rule;

pub(super) fn rules() -> Vec<Rule> {
    vec![Rule {
        identifier: "structure-extra-public-static-mut",
        title: "Public mutable static inventory",
        description: "Public mutable static inventory.",
        analysis: "Rust syntax and test-scoped token patterns",
        positive: "pub static mut STATE:u8=0;",
        negative: "pub static STATE:u8=0;",
    }]
}
