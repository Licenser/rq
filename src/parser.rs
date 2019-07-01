use crate::expr::*;
use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{digit1 as digit, multispace0 as multispace},
    combinator::{map, map_res},
    multi::many0,
    multi::separated_list,
    sequence::{delimited, preceded},
    IResult,
};

#[derive(Debug)]
pub enum Oper {
    Add,
    Sub,
    Mul,
    Div,
}

fn parens(i: &str) -> IResult<&str, Expr> {
    delimited(
        multispace,
        delimited(
            tag("("),
            map(math_expr, |e| Expr::Paren(Box::new(e))),
            tag(")"),
        ),
        multispace,
    )(i)
}

fn factor(i: &str) -> IResult<&str, Expr> {
    alt((
        map(
            map_res(delimited(multispace, digit, multispace), FromStr::from_str),
            Expr::Value,
        ),
        var_expr,
        parens,
    ))(i)
}

fn fold_exprs(initial: Expr, remainder: Vec<(Oper, Expr)>) -> Expr {
    remainder.into_iter().fold(initial, |acc, pair| {
        let (oper, expr) = pair;
        match oper {
            Oper::Add => Expr::Add(AddExpr {
                left: Box::new(acc),
                right: Box::new(expr),
            }),
            Oper::Sub => Expr::Sub(SubExpr {
                left: Box::new(acc),
                right: Box::new(expr),
            }),
            Oper::Mul => Expr::Mul(MulExpr {
                left: Box::new(acc),
                right: Box::new(expr),
            }),
            Oper::Div => Expr::Div(DivExpr {
                left: Box::new(acc),
                right: Box::new(expr),
            }),
        }
    })
}

fn term(i: &str) -> IResult<&str, Expr> {
    let (i, initial) = factor(i)?;
    let (i, remainder) = many0(alt((
        |i| {
            let (i, mul) = preceded(tag("*"), factor)(i)?;
            Ok((i, (Oper::Mul, mul)))
        },
        |i| {
            let (i, div) = preceded(tag("/"), factor)(i)?;
            Ok((i, (Oper::Div, div)))
        },
    )))(i)?;

    Ok((i, fold_exprs(initial, remainder)))
}

fn ident(i: &str) -> IResult<&str, String> {
    map(take_while1(move |c| c >= 'a' && c <= 'z'), String::from)(i)
}
fn math_expr(i: &str) -> IResult<&str, Expr> {
    let (i, initial) = term(i)?;
    let (i, remainder) = many0(alt((
        |i| {
            let (i, add) = preceded(tag("+"), term)(i)?;
            Ok((i, (Oper::Add, add)))
        },
        |i| {
            let (i, sub) = preceded(tag("-"), term)(i)?;
            Ok((i, (Oper::Sub, sub)))
        },
    )))(i)?;
    Ok((i, fold_exprs(initial, remainder)))
}

fn let_expr(i: &str) -> IResult<&str, Expr> {
    let (i, name) = preceded(tag("let"), delimited(multispace, ident, multispace))(i)?;
    let (i, expr) = preceded(tag("="), math_expr)(i)?;
    Ok((i, Expr::Let(name, Box::new(expr))))
}

fn var_expr(i: &str) -> IResult<&str, Expr> {
    map(delimited(multispace, ident, multispace), Expr::Var)(i)
}
fn expr(i: &str) -> IResult<&str, Expr> {
    alt((let_expr, math_expr))(i)
}

fn dl(i: &str) -> IResult<&str, &str> {
    preceded(multispace, tag(";"))(i)
}

pub fn exprs(i: &str) -> IResult<&str, Vec<Expr>> {
    separated_list(dl, preceded(multispace, expr))(i)
}
