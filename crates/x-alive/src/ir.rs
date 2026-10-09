use super::*;

const OPTIMIZER_PROBE_SOURCE: &str = r#"
#[no_mangle]
fn optimizer_probe(value: i32) -> i32 {
    let doubled: i32 = value * 2;
    return (doubled + 0) - value;
}
"#;

pub(super) fn compile_to_ir(
    source: &str,
    module_name: &str,
) -> Result<String, alive_messages::AliveError> {
    compile_to_ir_at_level(source, module_name, 0)
}

pub(super) fn compile_to_ir_at_level(
    source: &str,
    module_name: &str,
    current_optimisation_level: u8,
) -> Result<String, alive_messages::AliveError> {
    let context = Context::create();

    x_compile::compile_and_generate_ir(
        &context,
        source,
        module_name,
        IrCompileOptions {
            optimisation_level: current_optimisation_level,
            ..Default::default()
        },
    )
    .map(|generated| generated.ir)
    .map_err(alive_messages::compilation)
}

pub(super) fn stem_of(path: &Path) -> String {
    path.file_stem()
        .and_then(|text| text.to_str())
        .unwrap_or("main")
        .to_string()
}

pub(super) fn write_file(
    directory: &Path,
    name: &str,
    contents: &str,
) -> Result<PathBuf, alive_messages::AliveError> {
    fs::create_dir_all(directory)
        .map_err(|error| alive_messages::directory_create(directory.display(), error))?;
    let path = directory.join(name);
    fs::write(&path, contents)
        .map_err(|error| alive_messages::file_write(path.display(), error))?;

    Ok(path)
}

pub(super) fn normalize_ir_for_alive2(ir: &str) -> Result<String, alive_messages::AliveError> {
    // Strip X annotations only from the verifier copy.
    let x_attribute = Regex::new(r#"\s*"xlang\.[^"]+"="(?:\\.|[^"])*""#)
        .map_err(alive_messages::ir_normalizer)?;
    let mut normalized = x_attribute.replace_all(ir, "").into_owned();
    let borrow_attributes =
        Regex::new(r"\s+(?:noalias|nonnull)\b").map_err(alive_messages::borrow_normalizer)?;
    normalized = borrow_attributes.replace_all(&normalized, "").into_owned();
    let empty_group = Regex::new(r"(?m)^\s*attributes\s+(#[0-9]+)\s*=\s*\{\s*\}\s*$")
        .map_err(alive_messages::attribute_normalizer)?;
    let groups = empty_group
        .captures_iter(&normalized)
        .filter_map(|captures| captures.get(1).map(|group| group.as_str().to_string()))
        .collect::<Vec<_>>();
    normalized = empty_group.replace_all(&normalized, "").into_owned();
    for group in groups {
        normalized = normalized.replace(&format!(" {group} {{"), " {");
        let suffix = format!(" {group}");
        normalized = normalized
            .lines()
            .map(|line| line.strip_suffix(&suffix).unwrap_or(line))
            .collect::<Vec<_>>()
            .join("\n");
    }

    Ok(normalized)
}

pub(super) fn focal_contract_function(example_stem: &str) -> Option<&'static str> {
    match example_stem {
        "pure" => Some("square"),
        "unique" => Some("increment"),
        _ => None,
    }
}

pub(super) fn format_alive_proof(obligation: &ProofObligation, certificate: &str) -> String {
    format!(
        "obligation: {}\nsubject: {}\npredicate: {}\nchecker: alive2-process\nstatus: verified\n\n{}",
        obligation.identifier, obligation.subject, obligation.predicate, certificate,
    )
}

pub(super) fn verify_optimizer_probe(
    tool: &alive2::AliveTv,
    output_directory: &Path,
) -> Result<(), x_diagnostics::alive_messages::AliveError> {
    let probe_o0 = compile_to_ir_at_level(OPTIMIZER_PROBE_SOURCE, "x_alive_probe_o0", 0)?;
    let probe_o3 = compile_to_ir_at_level(OPTIMIZER_PROBE_SOURCE, "x_alive_probe_o3", 3)?;
    let probe_o0_path = write_file(
        output_directory,
        "optimizer-probe.o0.alive2.ll",
        &normalize_ir_for_alive2(&probe_o0)?,
    )?;
    let probe_o3_path = write_file(
        output_directory,
        "optimizer-probe.o3.alive2.ll",
        &normalize_ir_for_alive2(&probe_o3)?,
    )?;
    let result = run_alive_tv(tool, &probe_o0_path, &probe_o3_path, None)?;

    if !alive_success(&result) {
        return Err(alive_messages::optimizer_probe(format_alive_output(
            &result,
        )));
    }

    let obligation = ProofObligation::new(
        "x-alive optimizer probe",
        "the pinned LLVM 22 O3 pipeline refines generated scalar O0 IR",
    );
    let proof = format_alive_proof(&obligation, &format_alive_output(&result));
    let proof_path = write_file(output_directory, "optimizer-probe.alive2.proof.txt", &proof)?;
    println!("Alive2 optimiser probe OK; report {}", proof_path.display());

    Ok(())
}
