use super::inputs::valid_object_inputs;
use super::*;
use invocation::LinkInvocation;
use x_diagnostics::CompilationFailure;
use x_diagnostics::LinkDiagnostic;
use x_diagnostics::link_messages;

mod invocation;
pub(crate) use invocation::RuntimeSource;
