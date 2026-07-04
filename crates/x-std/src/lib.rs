#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

pub use x_runtime::RUNTIME_ABI_VERSION;

const MODULES: &[(&str, &str)] = &[
    ("std::ascii", include_str!("stdlib/stable/ascii.x")),
    ("std::bits", include_str!("stdlib/stable/bits.x")),
    ("std::io", include_str!("stdlib/stable/io.x")),
    ("std::math", include_str!("stdlib/stable/math.x")),
    ("std::num", include_str!("stdlib/stable/num.x")),
    ("std::process", include_str!("stdlib/stable/process.x")),
    ("std::slice", include_str!("stdlib/stable/slice.x")),
    ("std::str", include_str!("stdlib/stable/str.x")),
];

pub struct StdLibraryCatalog {
    files: &'static [(&'static str, &'static str)],
}

impl StdLibraryCatalog {
    pub fn new() -> Self {
        Self { files: MODULES }
    }

    pub fn get_module(&self, path: &str) -> Option<&'static str> {
        self.files
            .iter()
            .find_map(|(name, source)| (*name == path).then_some(*source))
    }

    pub fn module_names(&self) -> impl ExactSizeIterator<Item = &'static str> + '_ {
        self.files.iter().map(|(name, _)| *name)
    }

    pub fn all_modules(&self) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        self.files.iter().copied()
    }
}

impl Default for StdLibraryCatalog {
    fn default() -> Self {
        Self::new()
    }
}
