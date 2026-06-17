pub(super) fn rules() -> Vec<crate::Rule> {
    vec![
        crate::Rule {
            identifier: r####"abbreviation-variant"####,
            title: r####"Naming clarity / abbreviation-variant"####,
            description: r####"Different abbreviations for one configured term, such as cfg/config or diag/diagnostic."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){let cfg=1;let config=2;}"####,
            negative: r####"fn demo(){let config=1;}"####,
        },
        crate::Rule {
            identifier: r####"binding-definition-use-distance"####,
            title: r####"Naming clarity / binding-definition-use-distance"####,
            description: r####"A binding is declared well before its first use."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){let value=1;let other0=0;let other1=1;let other2=2;let other3=3;let other4=4;let other5=5;let other6=6;let other7=7;use_value(value);}"####,
            negative: r####"fn demo(){let value=1;let other=2;use_value(value);}"####,
        },
        crate::Rule {
            identifier: r####"binding-shadow-distance"####,
            title: r####"Naming clarity / binding-shadow-distance"####,
            description: r####"An inner binding shadows an outer value that is used again after the block."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){let owner=1;{let owner=2;}use_value(owner);}"####,
            negative: r####"fn demo(){let owner=1;{let owner=2;}}"####,
        },
        crate::Rule {
            identifier: r####"boilerplate-name-only-doc"####,
            title: r####"Comments and documentation / boilerplate-name-only-doc"####,
            description: r####"Rustdoc repeats the item name without describing its contract."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"/// read value
fn read_value(){}"####,
            negative: r####"/// Reads the validated value from the current record.
fn read_value(){}"####,
        },
        crate::Rule {
            identifier: r####"boolean-condition-density"####,
            title: r####"Control flow / boolean-condition-density"####,
            description: r####"A condition combines many Boolean operands or negations."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){if a&&b&&c&&d&&e{run();}}"####,
            negative: r####"fn demo(){if a&&b&&c&&d{run();}}"####,
        },
        crate::Rule {
            identifier: r####"closure-complexity"####,
            title: r####"Complexity and cohesion / closure-complexity"####,
            description: r####"An inline closure has many statements, branches or captures."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){let work=||{let value0=0;let value1=1;let value2=2;let value3=3;let value4=4;let value5=5;let value6=6;let value7=7;};}"####,
            negative: r####"fn demo(){let work=||{let first=1;let second=2;};}"####,
        },
        crate::Rule {
            identifier: r####"comment-contradicts-literal"####,
            title: r####"Comments and documentation / comment-contradicts-literal"####,
            description: r####"A comment states a limit, version or count that differs from the adjacent value."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"// limit: 12
const LIMIT:u32=13;"####,
            negative: r####"// limit: 12
const LIMIT:u32=12;"####,
        },
        crate::Rule {
            identifier: r####"commented-out-rust-block"####,
            title: r####"Comments and documentation / commented-out-rust-block"####,
            description: r####"A comment block parses as Rust statements or items."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"// let previous=12;
fn demo(){}"####,
            negative: r####"// The previous value has been retired.
fn demo(){}"####,
        },
        crate::Rule {
            identifier: r####"common-branch-prefix"####,
            title: r####"Duplication / common-branch-prefix"####,
            description: r####"Branches start with the same statement sequence."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){if check{begin();left();}else{begin();right();}}"####,
            negative: r####"fn demo(){if check{left();finish();}else{right();finish();}}"####,
        },
        crate::Rule {
            identifier: r####"common-branch-suffix"####,
            title: r####"Duplication / common-branch-suffix"####,
            description: r####"Branches end with the same statement sequence or return value."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){if check{left();finish();}else{right();finish();}}"####,
            negative: r####"fn demo(){if check{begin();left();}else{begin();right();}}"####,
        },
        crate::Rule {
            identifier: r####"confusable-name-pair"####,
            title: r####"Naming clarity / confusable-name-pair"####,
            description: r####"Nearby identifiers differ by one character, a transposition or a digit."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){let owner=1;let owners=2;}"####,
            negative: r####"fn demo(){let owner=1;let target=2;}"####,
        },
        crate::Rule {
            identifier: r####"copy-paste-field-outlier"####,
            title: r####"Duplication / copy-paste-field-outlier"####,
            description: r####"Similar field assignments contain one differing name or mapping."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){let value=Pair{left:left,right:left};}"####,
            negative: r####"fn demo(){let value=Pair{left:left,right:right};}"####,
        },
        crate::Rule {
            identifier: r####"decision-complexity-outlier"####,
            title: r####"Complexity and cohesion / decision-complexity-outlier"####,
            description: r####"A function has many decision points relative to its size."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){if condition0{run();}if condition1{run();}if condition2{run();}if condition3{run();}if condition4{run();}if condition5{run();}if condition6{run();}if condition7{run();}if condition8{run();}if condition9{run();}if condition10{run();}if condition11{run();}}"####,
            negative: r####"fn demo(){if condition0{run();}if condition1{run();}if condition2{run();}if condition3{run();}if condition4{run();}if condition5{run();}if condition6{run();}if condition7{run();}if condition8{run();}if condition9{run();}if condition10{run();}}"####,
        },
        crate::Rule {
            identifier: r####"deep-control-nesting"####,
            title: r####"Control flow / deep-control-nesting"####,
            description: r####"Control flow has excessive nesting."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){if a{if b{if c{if d{if e{run();}}}}}}"####,
            negative: r####"fn demo(){if a{if b{if c{if d{run();}}}}}"####,
        },
        crate::Rule {
            identifier: r####"duplicate-doc-contract-divergence"####,
            title: r####"Comments and documentation / duplicate-doc-contract-divergence"####,
            description: r####"Similar implementations have nearly identical docs with a changed contract sentence or literal."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"/// This operation returns exactly four values always.
fn first(){run();}
/// This operation returns exactly five values always.
fn second(){run();}"####,
            negative: r####"/// This operation returns exactly four values always.
fn first(){run();}
/// This operation returns exactly four values always.
fn second(){run();}"####,
        },
        crate::Rule {
            identifier: r####"duplicate-literal-table"####,
            title: r####"Duplication / duplicate-literal-table"####,
            description: r####"Substantial literal tables are repeated across Rust files."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){let first=[1,2,3,4,5,6,7,8];let second=[1,2,3,4,5,6,7,8];}"####,
            negative: r####"fn demo(){let first=[1,2,3,4,5,6,7,8];let second=[1,2,3,4,5,6,7,9];}"####,
        },
        crate::Rule {
            identifier: r####"duplicate-statement-window"####,
            title: r####"Duplication / duplicate-statement-window"####,
            description: r####"Repeated sequences contain at least three statements and eighteen tokens; overlapping windows count once."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn first(){let value=initialize(input);validate(&value)?;complete(value,options);}fn second(){let value=initialize(input);validate(&value)?;complete(value,options);}"####,
            negative: r####"fn first(){initialize();validate();complete();}fn second(){initialize();validate();cancel();}"####,
        },
        crate::Rule {
            identifier: r####"else-after-terminal-branch"####,
            title: r####"Control flow / else-after-terminal-branch"####,
            description: r####"An else branch follows an if branch that always exits."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){if check{return;}else{run();}}"####,
            negative: r####"fn demo(){if check{first();}else{second();}}"####,
        },
        crate::Rule {
            identifier: r####"empty-control-arm"####,
            title: r####"Control flow / empty-control-arm"####,
            description: r####"A control-flow branch is empty."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){if check{}}"####,
            negative: r####"fn demo(){if check{run();}}"####,
        },
        crate::Rule {
            identifier: r####"expression-chain-depth"####,
            title: r####"Complexity and cohesion / expression-chain-depth"####,
            description: r####"A long call chain or nested expression obscures value flow."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){value.a().b().c().d().e().f().g().h();}"####,
            negative: r####"fn demo(){value.a().b().c().d().e().f().g();}"####,
        },
        crate::Rule {
            identifier: r####"guard-condition-fragmentation"####,
            title: r####"Control flow / guard-condition-fragmentation"####,
            description: r####"Adjacent guards have the same exit."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){if first{return;}if second{return;}}"####,
            negative: r####"fn demo(){if first{return 1;}if second{return 2;}}"####,
        },
        crate::Rule {
            identifier: r####"identical-sibling-branch-body"####,
            title: r####"Duplication / identical-sibling-branch-body"####,
            description: r####"Different branches execute the same body."####,
            analysis: r####"Rust syntax, binding names, literals and attached rustdoc"####,
            positive: r####"fn demo(){if check{run();}else{run();}}"####,
            negative: r####"fn demo(){if check{first();}else{second();}}"####,
        },
    ]
}
