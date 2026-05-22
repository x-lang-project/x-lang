use crate::Error;

pub fn runtime_string_interpolation_required() -> String {
    Error::new("string interpolation requires runtime value lowering").render_detail()
}
