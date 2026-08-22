use super::*;

pub(super) fn validate_external_c_abi(name: &str, abi: Option<&str>) -> Result<(), TypeError> {
    let label = abi.map(|label| label.trim().trim_matches(['"', '\'']));

    if label.is_some_and(|label| label.eq_ignore_ascii_case("c")) {
        Ok(())
    } else {
        Err(TypeError::inference(
            FrontendMessage::UnsupportedExternAbi {
                name: name.to_string(),
                found: label.unwrap_or("no ABI").to_string(),
            },
        ))
    }
}
