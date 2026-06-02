use crate::Error;

pub fn incompatible_overflow_intrinsic_declaration(symbol: &str) -> String {
    Error::new(format!(
        "incompatible overflow intrinsic declaration: {symbol}"
    ))
    .render_detail()
}

pub fn global_name_has_no_registered_declaration(name: &str) -> String {
    Error::new(format!("global {name} has no registered declaration")).render_detail()
}
