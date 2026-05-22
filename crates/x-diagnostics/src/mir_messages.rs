use crate::Error;

pub fn catch_argument_count() -> String {
    Error::new("catch requires exactly a throwing call and fallback").render_detail()
}

pub fn missing_function_return(name: &str) -> String {
    Error::new(format!(
        "function `{name}` can finish without returning its declared value"
    ))
    .render_detail()
}

pub fn tuple_binding_arity() -> String {
    Error::new("tuple binding requires a checked tuple layout with matching arity").render_detail()
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
