mod calls;
mod errors;
mod types;

use crate::BlockIdentifier;
use crate::LocalIdentifier;
use crate::MirBody;

use types::validate_type;
use x_diagnostics::backend_messages;

pub use errors::MirValidationError;
