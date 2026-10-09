use super::*;

pub(super) fn run(cli: Cli) -> Result<(), CompilationFailure> {
    if cli.version {
        print_version();

        return Ok(());
    }

    match cli.command.ok_or_else(CliDiagnostic::command_missing)? {
        CommandKind::New { path } => {
            let created = x_driver::scaffold_new_project(&path)?;
            println!("created X project: {}", created.display());
            println!("next:");
            println!("  cd {}", created.display());
            println!("  x run .");

            Ok(())
        }
        CommandKind::Check { input } => check_path(&input),
        CommandKind::Build {
            input,
            json,
            output,
            optimisation_level,
            save_ir,
            emit_llvm,
            target,
            no_cache,
        } => build_path(
            &input,
            BuildOptions {
                output: output.as_deref(),
                optimisation_level,
                save_ir,
                emit_llvm,
                target: target.as_deref(),
                no_cache,
            },
            BuildAction::Build(if json {
                OutputFormat::Json
            } else {
                OutputFormat::Text
            }),
        ),
        CommandKind::CheriBuild {
            input,
            output,
            clang,
            sysroot,
            optimisation_level,
            save_ir,
        } => cheri::build_cheri_object(
            &input,
            &output,
            cheri::CheriBuildOptions {
                clang: &clang,
                sysroot: &sysroot,
                optimisation_level,
                save_ir,
            },
        ),
        CommandKind::Run {
            input,
            optimisation_level,
            save_ir,
            emit_llvm,
            no_cache,
            jit,
        } => {
            if jit {
                compilation::jit::run(&input, optimisation_level)
            } else {
                build_path(
                    &input,
                    BuildOptions {
                        optimisation_level,
                        save_ir,
                        emit_llvm,
                        no_cache,
                        ..BuildOptions::default()
                    },
                    BuildAction::Run,
                )
            }
        }
        CommandKind::JitWorker {
            input,
            optimisation_level,
        } => compilation::jit::worker(&input, optimisation_level),
        CommandKind::Doctor => doctor(),
    }
}
