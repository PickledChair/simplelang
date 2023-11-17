use super::expr::*;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Assign(Identifier, Expression),
    If(Expression, Box<Statement>),
    Print(Expression),
}
