use super::inputs::valid_object_inputs;
use super::*;
use invocation::LinkInvocation;
use x_diagnostics::CompilationFailure;
use x_diagnostics::LinkDiagnostic;
use x_diagnostics::link_messages;

mod invocation;
pub(crate) use invocation::RuntimeSource;

pub fn link(
    object_files: &[String],
    output_exe: &str,
    options: LinkOptions<'_>,
) -> Result<(), CompilationFailure> {
    let toolchain = NativeToolchain::discover()?;
    let clang = toolchain.compiler();
    let mut invocation = LinkInvocation::prepare(object_files, &options)?;
    invocation.reuse_runtime_object(&toolchain, &options);
    let rendered = invocation.render(output_exe, &options);

    if invocation.runtime_source().is_none()
        && let Some(result) = crate::fast_linker::try_link(
            &toolchain,
            crate::fast_linker::LinkRequest {
                arguments: &rendered.arguments,
                inputs: &rendered.inputs,
                output: output_exe,
                options: &options,
            },
        )
    {
        result?;

        return if Path::new(output_exe).is_file() {
            Ok(())
        } else {
            Err(link_messages::output_missing(output_exe))
        };
    }

    info!(
        "Invoking {} with args: {:?}",
        clang.display(),
        rendered.arguments
    );
    let mut command = toolchain.command();
    command.args(&rendered.arguments);
    if options.reproducible {
        command.env("SOURCE_DATE_EPOCH", "0");
    }

    let status = command.status().map_err(link_messages::clang_launch)?;

    if status.success() {
        if Path::new(output_exe).is_file() {
            Ok(())
        } else {
            Err(link_messages::output_missing(output_exe))
        }
    } else {
        Err(link_messages::clang_failure(status))
    }
}
