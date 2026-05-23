use crate::Error;

pub fn invariant_reference_context_required(detail: &str) -> String {
    Error::new(format!(
        "owner-dependent references require retained local mutation context for invariant enforcement: {detail}"
    ))
    .render_detail()
}

pub fn statement_aware_match_required() -> String {
    Error::new("matches require statement-aware lowering and resolved variant layouts")
        .render_detail()
}

pub fn checked_nominal_method_required() -> String {
    Error::new("nominal methods require a checked declaration and explicit receiver contract")
        .render_detail()
}

pub fn catch_argument_count() -> String {
    Error::new("catch requires exactly a throwing call and fallback").render_detail()
}

pub fn missing_function_return(name: &str) -> String {
    Error::new(format!(
        "function `{name}` can finish without returning its declared value"
    ))
    .render_detail()
}

pub fn resolved_match_layout_required() -> String {
    Error::new("refutable matches require a resolved discriminant and variant layout")
        .render_detail()
}

pub fn tuple_binding_arity() -> String {
    Error::new("tuple binding requires a checked tuple layout with matching arity").render_detail()
}

pub fn unsupported_static_initializer(name: &str) -> String {
    Error::new(format!(
        "static `{name}` initialiser requires a supported constant representation"
    ))
    .render_detail()
}

pub fn tuple_binding_layout() -> String {
    Error::new("tuple binding requires a checked tuple layout").render_detail()
}

pub fn missing_return_value() -> String {
    Error::new("nonvoid return requires its declared value").render_detail()
}

pub fn runtime_string_interpolation_required() -> String {
    Error::new("string interpolation requires runtime value lowering").render_detail()
}
