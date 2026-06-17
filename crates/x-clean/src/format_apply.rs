use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{Config, format_pipeline, inventory};
use x_diagnostics::tooling_messages::SourceReviewError;

struct Plan {
    path: PathBuf,
    original: String,
    formatted: String,
}

fn replace(path: &Path, text: &str, index: usize) -> Result<(), String> {
    let name = path
        .file_name()
        .ok_or_else(SourceReviewError::missing_source_filename)?
        .to_string_lossy();
    let temporary = path.with_file_name(format!(
        ".{name}.x-clean-{}-{index}.tmp",
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    let result = (|| {
        file.write_all(text.as_bytes())
            .map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        fs::set_permissions(
            &temporary,
            fs::metadata(path)
                .map_err(|error| error.to_string())?
                .permissions(),
        )
        .map_err(|error| error.to_string())?;
        drop(file);

        fs::rename(&temporary, path).map_err(|error| error.to_string())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }

    result
}

pub(crate) fn apply(root: &Path, config: &Config) -> Result<Vec<String>, String> {
    let root = root.canonicalize().map_err(|failure| failure.to_string())?;

    if !root.is_dir() || config.maximum_file_bytes == 0 {
        return Err(SourceReviewError::invalid_root_or_limit());
    }

    let mut originals = vec![];
    let mut input_paths = vec![];

    for path in inventory::paths(&root, config)? {
        let metadata = path
            .symlink_metadata()
            .map_err(|failure| failure.to_string())?;

        if metadata.file_type().is_symlink()
            || !path
                .canonicalize()
                .map_err(|failure| failure.to_string())?
                .starts_with(&root)
        {
            return Err(SourceReviewError::unsafe_source_path(path.display()));
        }

        if metadata.len() > config.maximum_file_bytes as u64 {
            return Err(SourceReviewError::source_too_large(path.display()));
        }

        let original = fs::read_to_string(&path).map_err(|failure| failure.to_string())?;
        let name = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        originals.push((name, original));
        input_paths.push(path);
    }

    let results = format_pipeline::compose(&originals)?;

    if format_pipeline::compose(&results)? != results {
        return Err(SourceReviewError::non_idempotent_pipeline());
    }

    let mut plans = vec![];

    for ((path, (_, original)), (_, formatted)) in
        input_paths.into_iter().zip(originals).zip(results)
    {
        if formatted != original {
            if fs::metadata(&path)
                .map_err(|failure| failure.to_string())?
                .permissions()
                .readonly()
            {
                return Err(SourceReviewError::source_read_only(path.display()));
            }

            plans.push(Plan {
                path,
                original,
                formatted,
            });
        }
    }

    for plan in &plans {
        if fs::read_to_string(&plan.path).map_err(|failure| failure.to_string())? != plan.original {
            return Err(SourceReviewError::source_changed_during_plan(
                plan.path.display(),
            ));
        }
    }

    let mut written: Vec<&Plan> = vec![];

    for (index, plan) in plans.iter().enumerate() {
        let result = (|| {
            if fs::read_to_string(&plan.path).map_err(|failure| failure.to_string())?
                != plan.original
            {
                return Err(SourceReviewError::source_changed_before_write(
                    plan.path.display(),
                ));
            }

            replace(&plan.path, &plan.formatted, index)
        })();

        if let Err(error) = result {
            let mut rollback_errors = vec![];

            for (rollback_index, prior) in written.iter().enumerate().rev() {
                if fs::read_to_string(&prior.path).is_ok_and(|text| text == prior.formatted) {
                    if let Err(error) = replace(&prior.path, &prior.original, rollback_index) {
                        rollback_errors.push(error);
                    }
                } else {
                    rollback_errors.push(SourceReviewError::concurrent_source_preserved(
                        prior.path.display(),
                    ));
                }
            }

            return Err(SourceReviewError::rollback_failed(
                error,
                rollback_errors.join("; "),
            ));
        }

        written.push(plan);
    }

    Ok(plans
        .into_iter()
        .map(|plan| {
            plan.path
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect())
}
