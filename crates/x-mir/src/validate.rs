mod errors;
mod types;

use crate::LocalIdentifier;

use types::validate_type;
use x_diagnostics::backend_messages;

pub use errors::MirValidationError;
