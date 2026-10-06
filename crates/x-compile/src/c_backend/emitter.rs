use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;

use super::error::EmitError;

use representation::*;
use x_ast::{Operator, Type, UnaryOperator, intrinsics::BuiltinKind};
use x_diagnostics::backend_messages;
use x_mir::*;

type Result<T> = std::result::Result<T, EmitError>;
#[derive(Clone)]
struct Value {
    code: String,
    value_type: Type,
}
struct Signature<'a> {
    symbol: String,
    parameters: &'a [Type],
    return_value: &'a Type,
    body: &'a MirBody,
}
struct Emitter<'a> {
    symbols: HashMap<&'a str, Signature<'a>>,
    order: Vec<&'a str>,
    helpers: BTreeSet<String>,
    structs: HashMap<&'a str, &'a x_ast::StructDefinition>,
    strings: RefCell<HashMap<String, usize>>,
}

mod representation;

mod values;
