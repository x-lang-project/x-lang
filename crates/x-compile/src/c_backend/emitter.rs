use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;

use super::error::EmitError;
use arithmetic::helper;
use representation::*;
use x_ast::{Operator, Type, UnaryOperator, intrinsics::BuiltinKind};
use x_diagnostics::backend_messages;
use x_mir::*;

mod declarations;
mod memoise;
mod registry;
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

mod arithmetic;
mod calls;
mod definitions;
mod representation;
mod rvalues;
mod terminators;
mod values;

pub(super) fn emit(program: &MirProgram) -> Result<String> {
    let mut error = Emitter {
        symbols: HashMap::new(),
        order: Vec::new(),
        helpers: BTreeSet::new(),
        structs: HashMap::new(),
        strings: RefCell::new(HashMap::new()),
    };
    error.register(&program.items)?;
    let mut declarations = error.type_declarations()?;
    let memo_globals = error.memoise_globals()?;
    error.declare_functions(&mut declarations)?;
    let mut bodies = String::new();

    for name in error.order.clone() {
        bodies.push_str(&error.definition(name)?)
    }

    let literals = error.string_literals()?;
    let mut output = String::from(
        "extern int printf(const char*,...);extern int fflush(void*);extern void exit(int);extern void x_runtime_write_stdout(const char*,unsigned long long);extern float fmodf(float,float);extern double fmod(double,double);static void _x_trap(void){__asm__(\"ud2\");}\n",
    );
    output.push_str(&declarations);
    if !memo_globals.is_empty() {
        output.push_str(memoise::SUPPORT);
        output.push_str(&memo_globals);
    }

    output.push_str(&literals);
    output.push_str("static unsigned long long _x_index(unsigned long long i,unsigned long long n){if(i>=n)_x_trap();return i;}\n");
    for h in &error.helpers {
        output.push_str(&helper(h)?)
    }

    output.push_str(&bodies);

    Ok(output)
}
