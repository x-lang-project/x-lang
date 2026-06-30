use crate::Type;

pub fn symbol_name_component(name: &str) -> String {
    let encoded = name
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    format!("{}_{encoded}", name.len())
}
