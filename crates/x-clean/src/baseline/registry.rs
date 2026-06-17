use crate::Rule;

fn rule(
    identifier: &'static str,
    description: &'static str,
    positive: &'static str,
    negative: &'static str,
) -> Rule {
    Rule {
        identifier,
        title: identifier,
        description,
        analysis: "syntax review candidate",
        positive,
        negative,
    }
}

pub fn rules() -> Vec<Rule> {
    vec![
        rule(
            "short-name",
            "Bindings no longer than two characters.",
            "fn f(){let i=1;}",
            "fn f(){let index=1;}",
        ),
        rule(
            "wildcard-binding",
            "Discard or wildcard pattern.",
            "fn f(){let _=1;}",
            "fn f(){let index=1;}",
        ),
        rule(
            "underscore-binding",
            "Underscore-prefixed binding.",
            "fn f(){let _owner=1;}",
            "fn f(){let owner=1;}",
        ),
        rule(
            "vague-name",
            "Temporary or generic binding vocabulary.",
            "fn f(){let temporary_tmp=1;}",
            "fn f(){let count=1;}",
        ),
        rule(
            "binding-naming-style",
            "Uppercase binding style.",
            "fn f(){let itemCount=1;}",
            "fn f(){let item_count=1;}",
        ),
        rule(
            "repeated-name-word",
            "A name repeats a word.",
            "fn count_count(){}",
            "fn count_items(){}",
        ),
        rule(
            "long-function",
            "Function exceeds 80 physical lines.",
            LONG,
            "fn f(){}",
        ),
        rule(
            "excessive-parameters",
            "Function has more than six nonreceiver parameters.",
            "fn f(aa:i32,bb:i32,cc:i32,dd:i32,ee:i32,ff:i32,gg:i32){}",
            "fn f(aa:i32){}",
        ),
        rule(
            "todo-marker",
            "Deferred-work comment marker.",
            "// TODO finish\nfn f(){}",
            "// implemented\nfn f(){}",
        ),
        rule(
            "review-debug-or-failure",
            "Debug output or explicit failure call.",
            "fn f(){panic!(\"reason\");}",
            "fn f(){finish();}",
        ),
        rule(
            "repeated-function-name",
            "Names reused across functions and methods.",
            "mod a{fn build(){}} mod b{fn build(){}}",
            "mod a{fn build(){}} mod b{fn finish(){}}",
        ),
        rule(
            "exact-function-body",
            "Duplicate substantive function token bodies.",
            EXACT,
            DIFFERENT,
        ),
        rule(
            "renamed-function-body",
            "Duplicate bodies after local-binding token renaming.",
            RENAMED,
            DIFFERENT,
        ),
        rule(
            "similar-function-body",
            "Similar substantive token sets; sparse search can miss pairs.",
            SIMILAR,
            DIFFERENT,
        ),
        rule(
            "duplicate-file",
            "Byte-identical files, excluding small forwarding and include wrappers.",
            EXACT,
            "fn different(){}",
        ),
        rule(
            "similarity-coverage-limit",
            "Approximate similarity skips common shingle buckets above 64 functions.",
            "fn example(){}",
            "fn example(){}",
        ),
    ]
}
const EXACT: &str = "fn a(){let count=1; step(count); step(count); step(count); step(count); step(count); step(count);} fn b(){let count=1; step(count); step(count); step(count); step(count); step(count); step(count);}";
const RENAMED: &str = "fn a(count:i32){let total=count+1; step(total); step(total); step(total); step(total); step(total);} fn b(size:i32){let sum=size+1; step(sum); step(sum); step(sum); step(sum); step(sum);}";
const SIMILAR: &str = "fn a(count:i32){let total=count+1; first(total); second(total); third(total); fourth(total); fifth(total); sixth(total); seventh(total); eighth(total); ninth(total); tenth(total); eleventh(total); twelfth(total); finish(2);} fn b(count:i32){let total=count+1; first(total); second(total); third(total); fourth(total); fifth(total); sixth(total); seventh(total); eighth(total); ninth(total); tenth(total); eleventh(total); twelfth(total); finish(3);}";
const DIFFERENT: &str = "fn a(){run();} fn b(){finish();}";
const LONG: &str = "fn f(){















































































}";
