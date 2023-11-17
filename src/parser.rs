use super::{expr::*, stmt::*};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric0, char, newline, one_of, space0, space1, u32},
    combinator::map,
    error::Error as NomError,
    multi::many0,
    sequence::tuple,
    Err as NomErr, IResult,
};

pub type Error<I> = NomErr<NomError<I>>;

fn identifier(s: &str) -> IResult<&str, Identifier> {
    let (rest, s0) = alpha1(s)?;
    map(alphanumeric0, |s1| (s0.to_owned() + s1).into())(rest)
}

fn number(s: &str) -> IResult<&str, Number> {
    map(u32, |n| n.into())(s)
}

fn term(s: &str) -> IResult<&str, Expression> {
    alt((
        map(identifier, |ident| Expression::Identifier(ident)),
        map(number, |num| Expression::Number(num)),
        map(
            tuple((char('('), space0, expression, space0, char(')'))),
            |(_, _, expr, _, _)| expr,
        ),
    ))(s)
}

fn addsub(s: &str) -> IResult<&str, Expression> {
    map(
        tuple((term, many0(tuple((space0, one_of("+-"), space0, term))))),
        |(t0, rest)| {
            let mut expr = t0;
            for (_, op, _, t1) in rest.into_iter() {
                match op {
                    '+' => expr = Expression::Add(Box::new(expr), Box::new(t1)),
                    '-' => expr = Expression::Sub(Box::new(expr), Box::new(t1)),
                    _ => unreachable!(),
                }
            }
            expr
        },
    )(s)
}

fn expression(s: &str) -> IResult<&str, Expression> {
    alt((
        map(
            tuple((addsub, space0, tag("=="), space0, addsub)),
            |(a1, _, _, _, a2)| Expression::Comp(Box::new(a1), Box::new(a2)),
        ),
        addsub,
    ))(s)
}

fn statement(s: &str) -> IResult<&str, Statement> {
    alt((
        map(
            tuple((identifier, space0, char('='), space0, expression, newline)),
            |(ident, _, _, _, expr, _)| Statement::Assign(ident, expr),
        ),
        map(
            tuple((
                tag("if"),
                space1,
                expression,
                space1,
                tag("then"),
                space1,
                statement,
            )),
            |(_, _, expr, _, _, _, stmt)| Statement::If(expr, Box::new(stmt)),
        ),
        map(
            tuple((tag("print"), space1, expression, newline)),
            |(_, _, expr, _)| Statement::Print(expr),
        ),
    ))(s)
}

pub fn parse(s: &str) -> Result<Statement, Error<&str>> {
    match statement(s) {
        Ok((_, stmt)) => Ok(stmt),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::{expr::*, stmt::*};

    #[test]
    fn parse_assign_statement_test() {
        let stmt = parse("answer1 = 42\n").unwrap();

        let expected = Statement::Assign(
            "answer1".to_owned().into(),
            Expression::Number(42u32.into()),
        );
        assert_eq!(stmt, expected, "expected {expected:?}, but got {stmt:?}");

        let stmt = parse("answer1 = answer2\n").unwrap();

        let expected = Statement::Assign(
            "answer1".to_owned().into(),
            Expression::Identifier("answer2".to_owned().into()),
        );
        assert_eq!(stmt, expected, "expected {expected:?}, but got {stmt:?}");
    }

    #[test]
    fn parse_print_statement_test() {
        let stmt = parse("print 42\n").unwrap();

        let expected = Statement::Print(Expression::Number(42u32.into()));
        assert_eq!(stmt, expected, "expected {expected:?}, but got {stmt:?}");

        let stmt = parse("print answer1\n").unwrap();

        let expected = Statement::Print(Expression::Identifier("answer1".to_owned().into()));
        assert_eq!(stmt, expected, "expected {expected:?}, but got {stmt:?}");
    }

    #[test]
    fn parse_if_statement_test() {
        let stmt = parse("if a == b then print 1\n").unwrap();

        let expected = Statement::If(
            Expression::Comp(
                Box::new(Expression::Identifier("a".to_owned().into())),
                Box::new(Expression::Identifier("b".to_owned().into())),
            ),
            Box::new(Statement::Print(Expression::Number(1u32.into()))),
        );
        assert_eq!(stmt, expected, "expected {expected:?}, but got {stmt:?}");
    }

    #[test]
    fn parser_binary_expression_test() {
        let stmt = parse("print 1 + 2 - 3\n").unwrap();

        let expected = Statement::Print(Expression::Sub(
            Box::new(Expression::Add(
                Box::new(Expression::Number(1u32.into())),
                Box::new(Expression::Number(2u32.into())),
            )),
            Box::new(Expression::Number(3u32.into())),
        ));
        assert_eq!(stmt, expected, "expected {expected:?}, but got {stmt:?}");
    }

    #[test]
    fn parser_group_expression_test() {
        let stmt = parse("print 1 + (2 - 3)\n").unwrap();

        let expected = Statement::Print(Expression::Add(
            Box::new(Expression::Number(1u32.into())),
            Box::new(Expression::Sub(
                Box::new(Expression::Number(2u32.into())),
                Box::new(Expression::Number(3u32.into())),
            )),
        ));
        assert_eq!(stmt, expected, "expected {expected:?}, but got {stmt:?}");
    }
}
