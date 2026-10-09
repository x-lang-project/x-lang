use std::fs;

use x_driver::{BuildArtifact, BuildOutput};

#[derive(Clone, Copy)]
pub(super) enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy)]
pub(super) struct RenderOptions {
    pub format: OutputFormat,
    pub emit_llvm: bool,
    pub cached: bool,
}

pub(super) fn render_build_output(artifact: &BuildOutput, options: RenderOptions) {
    match artifact {
        BuildOutput::Native(artifact) => render_build_artifact(artifact, options),
        BuildOutput::C(artifact) => render_c_artifact(artifact, options.format),
    }
}

fn render_c_artifact(artifact: &x_driver::CBuildArtifact, format: OutputFormat) {
    let compilation = &artifact.compilation;

    if matches!(format, OutputFormat::Json) {
        let debug = artifact
            .debug_artifacts
            .iter()
            .map(|artifact| serde_json::json!({"stage": artifact.stage, "text": artifact.text}))
            .collect::<Vec<_>>();
        let output = serde_json::json!({
            "schema_version": 2,
            "backend": "c",
            "path": artifact.path,
            "target_triple": artifact.target_triple,
            "executable": true,
            "metrics": {
                "build_seconds": artifact.elapsed_seconds,
                "c_emission_seconds": compilation.metrics.elapsed_seconds,
                "c_bytes": compilation.metrics.source_bytes,
                "c_lines": compilation.metrics.source_lines,
            },
            "debug": debug,
        });
        println!("{output}");
        return;
    }

    for debug in &artifact.debug_artifacts {
        println!("{} Representation Output:\n{}", debug.stage, debug.text);
    }

    println!("Build time: {:.6} s", artifact.elapsed_seconds);
    println!(
        "C: {} bytes, {} lines",
        compilation.metrics.source_bytes, compilation.metrics.source_lines
    );
    if let Ok(meta) = fs::metadata(&artifact.path) {
        println!("Executable: {} bytes", meta.len());
    }

    println!("Build successful: {}", artifact.path.display());
}

pub(super) fn render_build_artifact(artifact: &BuildArtifact, options: RenderOptions) {
    if matches!(options.format, OutputFormat::Json) {
        let compilation = &artifact.compilation;
        let debug = compilation
            .debug_artifacts
            .iter()
            .map(|artifact| serde_json::json!({"stage": artifact.stage, "text": artifact.text}))
            .collect::<Vec<_>>();
        let output = serde_json::json!({
            "schema_version": 1,
            "cached": options.cached,
            "path": artifact.path,
            "target_triple": compilation.target_triple,
            "executable": artifact.executable,
            "metrics": {
                "compile_seconds": compilation.metrics.elapsed_seconds,
                "ir_bytes": compilation.metrics.ir_bytes,
                "ir_lines": compilation.metrics.ir_lines,
            },
            "ir": if options.emit_llvm { Some(&compilation.ir) } else { None },
            "debug": debug,
        });

        println!("{output}");
        return;
    }

    let compilation = &artifact.compilation;

    for debug in &compilation.debug_artifacts {
        println!("{} Representation Output:\n{}", debug.stage, debug.text);
    }

    if options.emit_llvm {
        println!("{}", compilation.ir);
    }

    if options.cached {
        println!("Executable cache hit: compilation and linking skipped");
    }

    println!("Compile time: {:.6} s", compilation.metrics.elapsed_seconds);
    println!(
        "IR: {} bytes, {} lines",
        compilation.metrics.ir_bytes, compilation.metrics.ir_lines
    );

    if artifact.executable {
        if let Ok(meta) = fs::metadata(&artifact.path) {
            println!("Executable: {} bytes", meta.len());
        }

        println!("Build successful: {}", artifact.path.display());
    }
}
