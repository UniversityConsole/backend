use super::string_literal;
use super::types::{
    Expression, Field, FieldArg, FieldArgValue, SelectionSet, SingularSelectionSet,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::{take_while, take_while1};
use nom::character::complete::{char, multispace0};
use nom::character::{is_alphabetic, is_alphanumeric};
use nom::combinator::cut;
use nom::combinator::opt;
use nom::combinator::{map, recognize, value};
use nom::multi::separated_list0;
use nom::number::complete::double;
use nom::sequence::pair;
use nom::sequence::separated_pair;
use nom::sequence::tuple;
use nom::sequence::{preceded, terminated};
use nom::IResult;

pub fn bool(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("true")), value(false, tag("false"))))(input)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while1(|c| is_alphabetic(c as u8) || c == '_'),
        take_while(|c| is_alphanumeric(c as u8) || c == '_'),
    ))(input)
}

pub fn field_arg_value(input: &str) -> IResult<&str, FieldArgValue> {
    use string_literal::string_literal;

    alt((
        map(bool, FieldArgValue::BoolLiteral),
        map(double, FieldArgValue::NumericLiteral),
        map(string_literal, FieldArgValue::StringLiteral),
        map(tag("*"), |_| FieldArgValue::Wildcard),
    ))(input)
}

pub fn field_arg(input: &str) -> IResult<&str, FieldArg> {
    preceded(
        multispace0,
        map(
            separated_pair(
                identifier,
                tuple((multispace0, tag(":"), multispace0)),
                field_arg_value,
            ),
            |p: (&str, FieldArgValue)| FieldArg {
                name: p.0,
                value: p.1,
            },
        ),
    )(input)
}

pub fn field_args(input: &str) -> IResult<&str, Vec<FieldArg>> {
    preceded(
        char('('),
        cut(terminated(
            separated_list0(preceded(multispace0, char(',')), field_arg),
            preceded(multispace0, char(')')),
        )),
    )(input)
}

pub fn field(input: &str) -> IResult<&str, Field> {
    map(
        pair(identifier, opt(field_args)),
        |p: (&str, Option<Vec<FieldArg>>)| Field {
            name: p.0,
            args: p.1,
        },
    )(input)
}

pub fn singular_selection_set<'a>(input: &'a str) -> IResult<&'a str, SingularSelectionSet<'a>> {
    preceded(
        multispace0,
        alt((
            map(char('*'), |_| SingularSelectionSet::Wildcard),
            map(
                pair(field, opt(path_set)),
                |p: (Field, Option<Expression<'a>>)| {
                    SingularSelectionSet::Explicit(p.0, p.1.into())
                },
            ),
        )),
    )(input)
}

pub fn multi_selection_set<'a>(input: &'a str) -> IResult<&'a str, Vec<SingularSelectionSet<'a>>> {
    preceded(
        char('{'),
        cut(terminated(
            separated_list0(preceded(multispace0, char(',')), singular_selection_set),
            preceded(multispace0, char('}')),
        )),
    )(input)
}

pub fn selection_set<'a>(input: &'a str) -> IResult<&'a str, SelectionSet<'a>> {
    alt((
        map(singular_selection_set, SelectionSet::Singular),
        map(multi_selection_set, SelectionSet::Multi),
    ))(input)
}

pub fn path_set(input: &str) -> IResult<&str, Expression<'_>> {
    preceded(tag("::"), map(selection_set, Expression::SelectionSet))(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn playground() {
        println!("{:#?}", path_set("::{courses::{id, title, owner::*}, accounts(includeNonDiscoverable: true)::*, me::*}"));
    }
}
