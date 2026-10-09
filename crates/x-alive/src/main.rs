#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

mod alive2;
mod source_transform;

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use inkwell::context::Context;
use regex::Regex;
use walkdir::WalkDir;

use alive2::{
    alive_success, discover_alive_tv, format_alive_output, run_alive_tv, validate_alive_tv_version,
};
use comparison::*;
use ir::*;
use source_transform::strip_keywords;
use x_compile::IrCompileOptions;
use x_diagnostics::alive_messages;
use x_parser::parse;
use x_proofs::ProofObligation;

#[derive(Parser, Debug)]
#[command(name = "x-alive")]
#[command(about = "Generate LLVM IR pairs and verify them with Alive2")]
struct Cli {
    /// Directory containing .x examples
    #[arg(long, default_value = "./examples")]
    examples_directory: String,

    /// Output directory for .ll files
    #[arg(long, default_value = "./IR")]
    output_directory: String,

    /// Keywords to disable in the comparison variant
    #[arg(long = "keyword", short = 'k')]
    keywords: Vec<String>,

    /// Process files whose names contain this substring
    #[arg(long)]
    filter: Option<String>,

    /// Print the source and target Alive2 panes
    #[arg(long)]
    print_alive2: bool,

    /// Emit the paired .ll files without running Alive2
    #[arg(long)]
    emit_only: bool,

    /// Alive2 executable; Unix paths run through WSL on Windows
    #[arg(long)]
    alive_tv: Option<String>,

    /// WSL distribution for a Linux Alive2 executable
    #[arg(long, default_value = "Ubuntu-24.04")]
    wsl_distro: String,

    /// Keep processing after an Alive2 failure
    #[arg(long)]
    keep_going: bool,

    /// Print sources and parse trees while preparing each comparison
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Example files, relative to the examples directory
    #[arg(long = "file")]
    files: Vec<String>,
}

mod comparison;
mod ir;
const DEFAULT_FILES: &[&str] = &["conformance/pure.x", "conformance/unique.x"];

fn main() -> Result<(), alive_messages::AliveError> {
    x_logging::initialise().map_err(alive_messages::logging)?;

    run(Cli::parse())
}

fn run(mut arguments: Cli) -> Result<(), alive_messages::AliveError> {
    let examples_directory = PathBuf::from(&arguments.examples_directory);
    let output_directory = PathBuf::from(&arguments.output_directory);
    let alive_type_variable = if arguments.emit_only {
        None
    } else {
        discover_alive_tv(arguments.alive_tv.as_deref(), &arguments.wsl_distro)?
            .ok_or_else(alive_messages::tool_missing)?
            .into()
    };

    if let Some(tool) = &alive_type_variable {
        validate_alive_tv_version(tool)?;
    }

    if !examples_directory.exists() {
        return Err(alive_messages::examples_missing(
            examples_directory.display(),
        ));
    }

    let keywords = comparison_keywords(std::mem::take(&mut arguments.keywords));
    let mut totals = ComparisonTotals::default();

    if let Some(tool) = &alive_type_variable {
        verify_optimizer_probe(tool, &output_directory)?;
    }

    for path in example_entries(&examples_directory, &arguments.files)? {
        if selected_example(&path, arguments.filter.as_deref()) {
            totals.record(process_example(
                &arguments,
                &path,
                &keywords,
                alive_type_variable.as_ref(),
            )?);
        }
    }

    totals.finish(&arguments, &examples_directory, &output_directory)
}

const KW_TAG: &str = "semantics";

fn comparison_keywords(keywords: Vec<String>) -> Vec<String> {
    if keywords.is_empty() {
        vec![
            "pure".to_string(),
            "memoise".to_string(),
            "memoized".to_string(),
            "memoise".to_string(),
            "memoize".to_string(),
            "memo".to_string(),
            "invariant".to_string(),
            "parallel".to_string(),
            "static".to_string(),
            "throws".to_string(),
            "unique".to_string(),
            "become".to_string(),
        ]
    } else {
        keywords
    }
}

fn selected_example(path: &Path, filter: Option<&str>) -> bool {
    path.extension().is_some_and(|extension| extension == "x")
        && filter.is_none_or(|filter| path.to_string_lossy().contains(filter))
}

fn example_entries(
    examples_directory: &Path,
    files: &[String],
) -> Result<Vec<PathBuf>, x_diagnostics::alive_messages::AliveError> {
    Ok(if !files.is_empty() {
        files
            .iter()
            .map(|function| {
                let mut current_path = examples_directory.join(function);

                if current_path.extension().is_none() {
                    current_path.set_extension("x");
                }

                current_path
            })
            .collect()
    } else {
        let mut list = Vec::new();

        for function in DEFAULT_FILES {
            let current_path = examples_directory.join(function);

            if current_path.exists() {
                list.push(current_path);
            }
        }

        if !list.is_empty() {
            list
        } else {
            WalkDir::new(examples_directory)
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(alive_messages::example_enumeration)?
                .into_iter()
                .filter(|error| error.file_type().is_file())
                .map(|error| error.path().to_path_buf())
                .collect()
        }
    })
}
