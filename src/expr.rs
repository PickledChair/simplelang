use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Identifier(String);

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Identifier(value)
    }
}

impl Deref for Identifier {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct Number(u32);

impl From<u32> for Number {
    fn from(value: u32) -> Self {
        Number(value)
    }
}

impl From<Number> for u32 {
    fn from(value: Number) -> Self {
        value.0
    }
}

impl From<&Number> for u32 {
    fn from(value: &Number) -> Self {
        (*value).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Number(Number),
    Comp(Box<Expression>, Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
}
