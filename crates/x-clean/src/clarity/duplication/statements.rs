use std::cmp;

use std::collections::{BTreeMap, BTreeSet};

use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

use super::super::tokens;
use crate::{Finding, Source};

struct Block {
    source: usize,
    statements: Vec<String>,
    spans: Vec<proc_macro2::Span>,
}
struct Collector<'a> {
    source: usize,
    blocks: &'a mut Vec<Block>,
}
impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        if block.stmts.len() >= 3 {
            self.blocks.push(Block {
                source: self.source,
                statements: block.stmts.iter().map(tokens).collect(),
                spans: block.stmts.iter().map(Spanned::span).collect(),
            });
        }

        visit::visit_block(self, block);
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct Sequence {
    first: usize,
    first_start: usize,
    repeated: usize,
    repeated_start: usize,
    length: usize,
}

fn extend(blocks: &[Block], first: (usize, usize), repeated: (usize, usize)) -> Sequence {
    let (first_value, mut value) = first;
    let (second_value, mut other_value) = repeated;
    let left = &blocks[first_value].statements;
    let right = &blocks[second_value].statements;

    while value > 0
        && other_value > 0
        && left[value - 1] == right[other_value - 1]
        && (first_value != second_value || value + 3 <= other_value)
    {
        value -= 1;
        other_value -= 1;
    }

    let mut length = 3;

    while value + length < left.len()
        && other_value + length < right.len()
        && left[value + length] == right[other_value + length]
        && (first_value != second_value || value + length < other_value)
    {
        length += 1;
    }

    Sequence {
        first: first_value,
        first_start: value,
        repeated: second_value,
        repeated_start: other_value,
        length,
    }
}

fn substantive(statements: &[String]) -> bool {
    fn count(tokens: proc_macro2::TokenStream) -> usize {
        tokens
            .into_iter()
            .map(|token| match token {
                proc_macro2::TokenTree::Group(group) => 1 + count(group.stream()),
                _ => 1,
            })
            .sum()
    }
    statements
        .iter()
        .map(|text| count(text.parse().expect("AST statements are valid tokens")))
        .sum::<usize>()
        >= 18
        && statements.iter().collect::<BTreeSet<_>>().len() >= 2
}

pub(super) fn analyze(sources: &[Source], findings: &mut Vec<Finding>) {
    let mut blocks = vec![];

    for (source, data) in sources.iter().enumerate() {
        Collector {
            source,
            blocks: &mut blocks,
        }
        .visit_file(&data.ast);
    }

    let mut seeds = BTreeMap::<(bool, Vec<String>), Vec<(usize, usize)>>::new();
    let mut sequences = BTreeSet::new();

    for (block_index, block) in blocks.iter().enumerate() {
        let source = &sources[block.source];

        for (start, window) in block.statements.windows(3).enumerate() {
            let span = block.spans[start].start();
            let test = source.scope_at(span.line, span.column + 1) == crate::scope::Scope::Test;
            let sites = seeds.entry((test, window.to_vec())).or_default();
            // Retain earlier sites so a short match cannot hide a longer one.
            for &(first_block, first_start) in sites.iter() {
                if first_block == block_index && first_start + 3 > start {
                    continue;
                }

                let sequence = extend(&blocks, (first_block, first_start), (block_index, start));

                if substantive(
                    &block.statements
                        [sequence.repeated_start..sequence.repeated_start + sequence.length],
                ) {
                    sequences.insert(sequence);
                }
            }

            sites.push((block_index, start));
        }
    }

    let mut sequences = sequences.into_iter().collect::<Vec<_>>();
    sequences.sort_by_key(|sequence| (cmp::Reverse(sequence.length), *sequence));
    let mut accepted: Vec<Sequence> = vec![];

    for sequence in sequences {
        if accepted.iter().any(|other| {
            other.repeated == sequence.repeated
                && other.repeated_start < sequence.repeated_start + sequence.length
                && sequence.repeated_start < other.repeated_start + other.length
        }) {
            continue;
        }

        accepted.push(sequence);
    }

    accepted.sort_by_key(|sequence| (sequence.repeated, sequence.repeated_start));

    for sequence in accepted {
        let first = &blocks[sequence.first];
        let repeated = &blocks[sequence.repeated];
        let first_start = first.spans[sequence.first_start].start();
        let first_end = first.spans[sequence.first_start + sequence.length - 1].end();
        let start = repeated.spans[sequence.repeated_start].start();
        let end = repeated.spans[sequence.repeated_start + sequence.length - 1].end();
        let source = &sources[repeated.source];
        source.emit(
            findings,
            "duplicate-statement-window",
            source.position_offset(start.line, start.column),
            format!(
                "Repeated sequence of {} statements: {}:{}:{}–{}:{}, then {}:{}:{}–{}:{}.",
                sequence.length,
                sources[first.source].path,
                first_start.line,
                first_start.column + 1,
                first_end.line,
                first_end.column + 1,
                source.path,
                start.line,
                start.column + 1,
                end.line,
                end.column + 1
            ),
        );
    }
}
