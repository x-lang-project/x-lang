mod inference;
pub(super) use inference::infer_declared_type_substitutions;

use super::*;
use x_ast::GenericParameter;
use x_ast::TypeVisitorMut;
