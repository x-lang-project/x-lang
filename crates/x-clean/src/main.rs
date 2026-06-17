use std::env;
use std::fs;
use std::process;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use x_clean::{Config, apply_format, rules, scan};
use x_diagnostics::tooling_messages::SourceReviewError;

#[derive(Clone, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Parser)]
#[command(about = "Rust source review and formatting")]
struct Arguments {
    #[arg(default_value = ".")]
    root: PathBuf,
    #[arg(long)]
    list_rules: bool,
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long, value_enum)]
    profile: Option<x_clean::profile::Profile>,
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long)]
    text_output: Option<PathBuf>,
    #[arg(long)]
    fail_on_findings: bool,
    #[arg(long, conflicts_with = "list_rules")]
    write_format: bool,
    #[arg(long, conflicts_with = "skip_format")]
    check_format: bool,
    #[arg(long)]
    skip_format: bool,
}

fn execute(arguments: Arguments) -> Result<i32, String> {
    if arguments.list_rules {
        let rules = rules();

        match arguments.format {
            Format::Json => println!(
                "{}",
                serde_json::to_string_pretty(&rules).map_err(|error| error.to_string())?
            ),
            Format::Text => {
                for rule in rules {
                    println!(
                        "{}: {} [{}]",
                        rule.identifier, rule.description, rule.analysis
                    );
                }
            }
        }

        return Ok(0);
    }

    let root = arguments
        .root
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let mut config = if let Some(path) = arguments.config {
        serde_json::from_str(&fs::read_to_string(path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?
    } else {
        Config::default()
    };

    if let Some(profile) = arguments.profile {
        config.profile = profile;
    }

    if !arguments.only.is_empty() {
        config.only_rules = arguments.only;
    }

    if arguments.skip_format {
        config.check_format = false;
    }

    if arguments.check_format {
        config.check_format = true;
    }

    if arguments.write_format {
        if arguments.output.is_some() || arguments.text_output.is_some() {
            return Err(SourceReviewError::output_in_write_mode());
        }

        let changed = apply_format(&root, &config)?;

        match arguments.format {
            Format::Json => println!(
                "{}",
                serde_json::json!({"read_only": false, "formatted_files": changed})
            ),
            Format::Text => {
                println!("Formatted {} Rust files", changed.len());

                for path in changed {
                    println!("{path}");
                }
            }
        }

        return Ok(0);
    }

    let output_path = resolve_output(&root, arguments.output)?;
    let text_output_path = resolve_output(&root, arguments.text_output)?;

    if output_path.is_some() && output_path == text_output_path {
        return Err(SourceReviewError::duplicate_report_paths());
    }

    let report = scan(&root, &config)?;
    let rendered = match arguments.format {
        Format::Json => serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?,
        Format::Text => x_clean::scope::render_report(&report),
    };

    if let Some(path) = output_path {
        write_report(path, &rendered)?;
    } else {
        println!("{rendered}");
    }

    if let Some(path) = text_output_path {
        write_report(path, &x_clean::reporting::render(&report))?;
    }

    Ok(if !report.errors.is_empty() {
        2
    } else if arguments.fail_on_findings && !report.findings.is_empty() {
        1
    } else {
        0
    })
}

fn resolve_output(
    root: &std::path::Path,
    output: Option<PathBuf>,
) -> Result<Option<PathBuf>, String> {
    if let Some(path) = output {
        let absolute = if path.is_absolute() {
            path
        } else {
            env::current_dir()
                .map_err(|error| error.to_string())?
                .join(path)
        };

        let parent = absolute
            .parent()
            .ok_or_else(SourceReviewError::output_parent_missing)?
            .canonicalize()
            .map_err(SourceReviewError::output_directory_missing)?;

        if parent.starts_with(root) {
            return Err(SourceReviewError::output_inside_source());
        }

        let destination = parent.join(
            absolute
                .file_name()
                .ok_or_else(SourceReviewError::output_filename_missing)?,
        );

        if destination.exists() {
            return Err(SourceReviewError::output_exists());
        }

        Ok(Some(destination))
    } else {
        Ok(None)
    }
}

fn write_report(path: PathBuf, rendered: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| error.to_string())?;

    writeln!(file, "{rendered}").map_err(|error| error.to_string())
}

fn main() {
    let code = match execute(Arguments::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("x-clean: {error}");

            2
        }
    };

    process::exit(code);
}
