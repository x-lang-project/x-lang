use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use paths::*;
use x_diagnostics::alive_messages;
use x_utils::tools::LLVM_VERSION as REQUIRED_ALIVE_LLVM_VERSION;
use x_utils::tools::find_on_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AliveTv {
    Native(PathBuf),
    Wsl { distro: String, path: String },
}

impl AliveTv {
    fn command(&self) -> Command {
        match self {
            Self::Native(path) => Command::new(path),
            Self::Wsl { distro, path } => {
                let mut command = wsl_command(distro);
                command.arg(path);

                command
            }
        }
    }
}

fn output_text(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AliveSummary {
    pub(super) correct: usize,
    pub(super) incorrect: usize,
    pub(super) failed_to_prove: usize,
    pub(super) errors: usize,
}

impl AliveSummary {
    fn is_clean(&self) -> bool {
        self.correct > 0 && self.incorrect == 0 && self.failed_to_prove == 0 && self.errors == 0
    }
}

#[derive(Debug)]
pub(super) struct AliveResult {
    output: Output,
    pub(super) summary: Option<AliveSummary>,
}

fn executable_exists(path: &Path) -> bool {
    path.is_file()
}

fn parse_alive_tv(value: &str, distro: &str) -> AliveTv {
    if cfg!(windows) && value.starts_with('/') {
        AliveTv::Wsl {
            distro: distro.to_string(),
            path: value.to_string(),
        }
    } else {
        AliveTv::Native(PathBuf::from(value))
    }
}

pub(super) fn discover_alive_tv(
    explicit: Option<&str>,
    distro: &str,
) -> Result<Option<AliveTv>, alive_messages::AliveError> {
    if let Some(path) = explicit {
        return Ok(Some(parse_alive_tv(path, distro)));
    }

    for key in ["X_ALIVE_TV", "ALIVE_TV"] {
        if let Ok(path) = env::var(key)
            && !path.trim().is_empty()
        {
            return Ok(Some(parse_alive_tv(&path, distro)));
        }
    }

    if let Some(path) = find_on_path("alive-tv") {
        return Ok(Some(AliveTv::Native(path)));
    }

    for candidate in [
        PathBuf::from(".tools/alive2/build/alive-tv.exe"),
        PathBuf::from(".tools/alive2/build/alive-tv"),
    ] {
        if executable_exists(&candidate) {
            return Ok(Some(AliveTv::Native(candidate)));
        }
    }

    Ok(None)
}

fn parse_alive_llvm_version(output: &str) -> Option<&str> {
    output.lines().find_map(|line| {
        let (_, version) = line.trim().split_once("LLVM version ")?;

        version.split_whitespace().next()
    })
}

pub(super) fn validate_alive_tv_version(tool: &AliveTv) -> Result<(), alive_messages::AliveError> {
    let output = tool
        .command()
        .arg("--version")
        .output()
        .map_err(|error| match tool {
            AliveTv::Native(path) => alive_messages::tool_query(path.display(), error),
            AliveTv::Wsl { distro, .. } => alive_messages::wsl_tool_query(distro, error),
        })?;

    if !output.status.success() {
        return Err(alive_messages::version_query_failure(
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
        ));
    }

    let text = output_text(&output);
    let actual = parse_alive_llvm_version(&text)
        .ok_or_else(|| alive_messages::version_unparseable(&text))?;

    if actual != REQUIRED_ALIVE_LLVM_VERSION {
        return Err(alive_messages::version_mismatch(
            actual,
            REQUIRED_ALIVE_LLVM_VERSION,
        ));
    }

    Ok(())
}

pub(super) fn run_alive_tv(
    tool: &AliveTv,
    source: &Path,
    target: &Path,
    function: Option<&str>,
) -> Result<AliveResult, alive_messages::AliveError> {
    let (source, target) = match tool {
        AliveTv::Native(_) => (source.to_owned(), target.to_owned()),
        AliveTv::Wsl { distro, .. } => (
            PathBuf::from(path_to_wsl(distro, source)?),
            PathBuf::from(path_to_wsl(distro, target)?),
        ),
    };

    let mut command = tool.command();

    if let Some(function) = function {
        command.arg(format!("--func={function}"));
    }

    let output = command
        .arg(source)
        .arg(target)
        .output()
        .map_err(|error| match tool {
            AliveTv::Native(path) => alive_messages::tool_launch(path.display(), error),
            AliveTv::Wsl { distro, .. } => alive_messages::wsl_tool_launch(distro, error),
        })?;

    let text = output_text(&output);

    Ok(AliveResult {
        output,
        summary: parse_alive_summary(&text),
    })
}

fn parse_summary_count(line: &str, label: &str) -> Option<usize> {
    let trimmed = line.trim();
    let rest = trimmed.strip_suffix(label)?;

    rest.trim().parse().ok()
}

fn parse_alive_summary(text: &str) -> Option<AliveSummary> {
    let mut correct = None;
    let mut incorrect = None;
    let mut failed_to_prove = None;
    let mut errors = None;

    for line in text.lines() {
        correct = correct.or_else(|| parse_summary_count(line, "correct transformations"));
        incorrect = incorrect.or_else(|| parse_summary_count(line, "incorrect transformations"));
        failed_to_prove = failed_to_prove
            .or_else(|| parse_summary_count(line, "failed-to-prove transformations"));
        errors = errors.or_else(|| parse_summary_count(line, "Alive2 errors"));
    }

    Some(AliveSummary {
        correct: correct?,
        incorrect: incorrect?,
        failed_to_prove: failed_to_prove?,
        errors: errors?,
    })
}

pub(super) fn alive_success(result: &AliveResult) -> bool {
    result.output.status.success() && result.summary.as_ref().is_some_and(AliveSummary::is_clean)
}

pub(super) fn format_alive_output(result: &AliveResult) -> String {
    let stdout = String::from_utf8_lossy(&result.output.stdout);
    let stderr = String::from_utf8_lossy(&result.output.stderr);

    if stderr.trim().is_empty() {
        stdout.to_string()
    } else if stdout.trim().is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    }
}

mod paths;
