use crate::Error;

pub fn call_requires_resolved_return_type(function: &str) -> String {
    Error::new(format!("{function} requires a resolved return type")).render_detail()
}

pub fn print_requires_declared_value_function() -> String {
    Error::new("print returns void; a value result requires a declared function").render_detail()
}

pub fn dangling_requires_resolved_pointer_element() -> String {
    Error::new("dangling requires a resolved pointer element type").render_detail()
}

pub fn is_null_requires_typed_pointer() -> String {
    Error::new("is_null requires a typed pointer").render_detail()
}

pub fn call_requires_one_primitive_string_receiver(function: &str) -> String {
    Error::new(format!("{function} requires one primitive string receiver")).render_detail()
}

pub fn call_requires_primitive_string_receiver(function: &str) -> String {
    Error::new(format!("{function} requires a primitive string receiver")).render_detail()
}

pub fn incompatible_overflow_intrinsic_declaration(symbol: &str) -> String {
    Error::new(format!(
        "incompatible overflow intrinsic declaration: {symbol}"
    ))
    .render_detail()
}

pub fn call_requires_resolved_pointer_element(function: &str) -> String {
    Error::new(format!(
        "{function} requires a resolved pointer element type"
    ))
    .render_detail()
}

pub fn become_requires_identical_abi(caller: &str, callee: &str) -> String {
    Error::new(format!(
        "become requires identical caller/callee ABI: {caller} -> {callee}"
    ))
    .render_detail()
}

pub fn global_name_has_no_registered_declaration(name: &str) -> String {
    Error::new(format!("global {name} has no registered declaration")).render_detail()
}

pub fn slice_view_requires_explicit_identity_lifetime_and_mutability(name: &str) -> String {
    Error::new(format!(
        "{name} requires explicit view or iterator identity, lifetime, and mutability metadata"
    ))
    .render_detail()
}
