use super::*;

pub(super) fn validate_build(path: &Path, build: &toml::Value) -> Result<(), CompilationFailure> {
    let build = build
        .as_table()
        .ok_or_else(|| ProjectError::build_table_required(path.display()))?;
    reject_unknown_keys(
        path,
        "build",
        build,
        &["freestanding", "no_borrow_check", "optimization"],
    )?;
    for key in ["freestanding", "no_borrow_check"] {
        if build
            .get(key)
            .is_some_and(|value| value.as_bool().is_none())
        {
            return Err(ProjectError::build_boolean_required(path.display(), key));
        }
    }

    if build.get("optimization").is_some_and(|value| {
        value
            .as_integer()
            .is_none_or(|level| !(0..=i64::from(NativeOptimization::MAX)).contains(&level))
    }) {
        return Err(ProjectError::build_optimization_invalid(path.display()));
    }

    if !build
        .get("freestanding")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
    {
        return Err(ProjectError::freestanding_required(path.display()));
    }

    Ok(())
}
