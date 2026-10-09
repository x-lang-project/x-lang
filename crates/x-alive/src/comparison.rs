use super::*;

pub(super) enum ComparisonOutcome {
    ParseFailed,
    Emitted,
    Verified,
    VerificationFailed,
}

#[derive(Default)]
pub(super) struct ComparisonTotals {
    processed: usize,
    verified: usize,
    failed: usize,
}

impl ComparisonTotals {
    pub(super) fn record(&mut self, outcome: ComparisonOutcome) {
        match outcome {
            ComparisonOutcome::ParseFailed => self.failed += 1,
            ComparisonOutcome::Emitted => self.processed += 1,
            ComparisonOutcome::Verified => {
                self.processed += 1;
                self.verified += 1;
            }
            ComparisonOutcome::VerificationFailed => {
                self.processed += 1;
                self.failed += 1;
            }
        }
    }

    pub(super) fn finish(
        self,
        arguments: &Cli,
        examples_directory: &Path,
        output_directory: &Path,
    ) -> Result<(), alive_messages::AliveError> {
        if self.failed > 0 {
            return Err(alive_messages::comparisons_failed(self.failed));
        }

        if self.processed == 0 {
            return Err(alive_messages::no_comparisons(examples_directory.display()));
        }

        if arguments.emit_only {
            println!(
                "Files processed: {}; output: {}",
                self.processed,
                output_directory.display()
            );
        } else {
            println!(
                "Files: {}; verified: {}; failed: {}; output: {}",
                self.processed,
                self.verified,
                self.failed,
                output_directory.display()
            );
        }

        Ok(())
    }
}

pub(super) fn process_example(
    arguments: &Cli,
    path: &Path,
    keywords: &[String],
    alive_type_variable: Option<&alive2::AliveTv>,
) -> Result<ComparisonOutcome, alive_messages::AliveError> {
    let Some(comparison) = IrComparison::prepare(arguments, path, keywords)? else {
        return Ok(ComparisonOutcome::ParseFailed);
    };

    let output_directory = Path::new(&arguments.output_directory);
    comparison.write_artifacts(output_directory, arguments.print_alive2)?;

    match alive_type_variable {
        None => Ok(ComparisonOutcome::Emitted),
        Some(tool) if comparison.verify(tool, path, arguments)? => Ok(ComparisonOutcome::Verified),
        Some(_) => Ok(ComparisonOutcome::VerificationFailed),
    }
}

struct IrComparison {
    name: String,
    ir_with: String,
    ir_without: String,
}

impl IrComparison {
    pub(super) fn prepare(
        arguments: &Cli,
        path: &Path,
        keywords: &[String],
    ) -> Result<Option<Self>, alive_messages::AliveError> {
        let source = fs::read_to_string(path)
            .map_err(|failure| alive_messages::file_read(path.display(), failure))?;

        let name = stem_of(path);
        println!("=== Processing {} ===", path.display());

        if arguments.verbose {
            println!(
                "--- Original source ({}) ---\n{}\n--- end original ---",
                path.display(),
                source
            );

            match parse(&source) {
                Ok(pointer) => println!("Parse AST (original): {pointer:#?}"),
                Err(failure) => println!("Parse error (original): {failure}\n"),
            }
        }

        let ir_with = compile_to_ir(&source, &name)
            .map_err(|failure| alive_messages::compile_original(path.display(), failure))?;

        let source_without = strip_keywords(&source, keywords)?;

        match parse(&source_without) {
            Ok(pointer) => {
                if arguments.verbose {
                    println!(
                        "--- Stripped source ({}) ---\n{}\n--- end stripped ---",
                        path.display(),
                        source_without
                    );
                    println!("Parse AST (stripped): {pointer:#?}");
                }
            }
            Err(failure) => {
                let error = alive_messages::parse_comparison(path.display(), failure);

                if !arguments.keep_going {
                    return Err(error);
                }

                eprintln!("{error}");

                return Ok(None);
            }
        }

        let ir_without =
            compile_to_ir(&source_without, &format!("{name}_no_{KW_TAG}")).map_err(|failure| {
                alive_messages::compile_comparison(KW_TAG, path.display(), failure)
            })?;

        Ok(Some(Self {
            name,
            ir_with,
            ir_without,
        }))
    }

    pub(super) fn write_artifacts(
        &self,
        output_directory: &Path,
        print_alive2: bool,
    ) -> Result<(), alive_messages::AliveError> {
        let Self {
            name,
            ir_with,
            ir_without,
        } = self;
        let with_name = format!("{name}.with.{KW_TAG}.ll");
        let without_name = format!("{name}.without.{KW_TAG}.ll");

        let output_with = write_file(output_directory, &with_name, ir_with)?;
        let output_without = write_file(output_directory, &without_name, ir_without)?;

        println!(
            "  -> wrote:\n     {}\n     {}",
            output_with.display(),
            output_without.display()
        );

        if print_alive2 {
            println!("\n----- Alive2 Source (before) [{name}] -----\n{ir_without}");
            println!("\n----- Alive2 Target (after)  [{name}] -----\n{ir_with}\n");
        }

        Ok(())
    }

    pub(super) fn verify(
        &self,
        tool: &alive2::AliveTv,
        path: &Path,
        arguments: &Cli,
    ) -> Result<bool, alive_messages::AliveError> {
        let Self {
            name,
            ir_with,
            ir_without,
        } = self;
        let output_directory = Path::new(&arguments.output_directory);
        let alive_with_name = format!("{name}.with.{KW_TAG}.alive2.ll");
        let alive_without_name = format!("{name}.without.{KW_TAG}.alive2.ll");
        let alive_with = write_file(
            output_directory,
            &alive_with_name,
            &normalize_ir_for_alive2(ir_with)?,
        )?;
        let alive_without = write_file(
            output_directory,
            &alive_without_name,
            &normalize_ir_for_alive2(ir_without)?,
        )?;
        let focal_function = focal_contract_function(name);
        let result = run_alive_tv(tool, &alive_without, &alive_with, focal_function)?;

        if alive_success(&result) {
            let certificate = format_alive_output(&result);
            let obligation = ProofObligation::new(
                path.to_string_lossy(),
                format!(
                    "LLVM target focal definition with `{KW_TAG}` refines the keyword-stripped LLVM source after erasing X-only strings and Alive2-unsupported borrow attributes"
                ),
            );
            let proof_text = format_alive_proof(&obligation, &certificate);
            let proof_name = format!("{name}.{KW_TAG}.alive2.proof.txt");
            let proof_path = write_file(output_directory, &proof_name, &proof_text)?;

            if let Some(summary) = &result.summary {
                println!(
                    "  -> Alive2 OK: {} correct, {} incorrect, {} failed-to-prove, {} errors; report {}",
                    summary.correct,
                    summary.incorrect,
                    summary.failed_to_prove,
                    summary.errors,
                    proof_path.display(),
                );
            } else {
                println!("  -> Alive2 OK");
            }

            Ok(true)
        } else {
            let output = format_alive_output(&result);
            println!("  -> Alive2 FAILED for {name}\n{output}");
            if !arguments.keep_going {
                return Err(alive_messages::verification(path.display()));
            }

            Ok(false)
        }
    }
}
