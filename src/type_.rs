#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeRepr {
    Int,
    Bool,
    Id(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Var(usize, Option<TypeRepr>),
}
