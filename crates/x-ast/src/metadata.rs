#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lifetime {
    Static,
    Named(String),
    Infer(usize),
    Erased,
}
