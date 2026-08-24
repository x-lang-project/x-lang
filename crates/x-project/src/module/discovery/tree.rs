use super::*;

struct PendingModule {
    name: String,
    path: PathBuf,
    depth: usize,
}

pub(in crate::module) struct Discovery {
    pub(in crate::module) modules: HashMap<String, ModuleNode>,
    pub(in crate::module) parsed_modules: HashMap<String, x_parser::ParsedModule>,
    pub(in crate::module) total_source_bytes: usize,
}
