#[derive(Debug, Clone, Copy, Default)]
pub struct IrCompileOptions<'a> {
    pub show_parse: bool,
    pub optimisation_level: u8,
    pub target_triple: Option<&'a str>,
    pub dce: bool,
    pub inline_budget: u32,
    pub optimizer_diagnostics: bool,
    pub discard_value_names: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ObjectCompileOptions<'a> {
    pub ir: IrCompileOptions<'a>,
    pub save_ir: bool,
}
