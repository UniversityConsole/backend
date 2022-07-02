use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{char, digit1, multispace0, one_of};
use nom::character::{is_alphabetic, is_alphanumeric};
use nom::combinator::{cut, eof, map, map_res, opt, recognize, value};
use nom::multi::separated_list0;
use nom::sequence::{pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;

use super::string_literal;
use super::types::{Expression, Field, FieldArg, FieldArgValue, SelectionSet, SingularSelectionSet};

pub fn bool(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("true")), value(false, tag("false"))))(input)
}

pub fn i64(input: &str) -> IResult<&str, i64> {
    map_res(recognize(pair(opt(one_of("+-")), digit1)), str::parse)(input)
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
        map(i64, FieldArgValue::IntegerLiteral),
        map(string_literal, FieldArgValue::StringLiteral),
        map(identifier, |s| FieldArgValue::Enum(s.to_owned())),
        map(tag("*"), |_| FieldArgValue::Wildcard),
    ))(input)
}

pub fn field_arg(input: &str) -> IResult<&str, FieldArg> {
    preceded(
        multispace0,
        map(
            separated_pair(identifier, tuple((multispace0, tag(":"), multispace0)), field_arg_value),
            |p: (&str, FieldArgValue)| FieldArg { name: p.0, value: p.1 },
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
    map(pair(identifier, opt(field_args)), |p: (&str, Option<Vec<FieldArg>>)| {
        Field { name: p.0, args: p.1 }
    })(input)
}

pub fn singular_selection_set<'a>(input: &'a str) -> IResult<&'a str, SingularSelectionSet<'a>> {
    preceded(
        multispace0,
        alt((
            map(char('*'), |_| SingularSelectionSet::Wildcard),
            map(
                pair(field, opt(preceded(tag("::"), selection_set))),
                |p: (Field, Option<SelectionSet<'a>>)| SingularSelectionSet::Explicit(p.0, p.1.into()),
            ),
        )),
    )(input)
}

pub fn multi_selection_set(input: &str) -> IResult<&str, Vec<SingularSelectionSet<'_>>> {
    preceded(
        char('{'),
        cut(terminated(
            separated_list0(preceded(multispace0, char(',')), singular_selection_set),
            preceded(multispace0, char('}')),
        )),
    )(input)
}

pub fn selection_set(input: &str) -> IResult<&str, SelectionSet<'_>> {
    alt((
        map(singular_selection_set, SelectionSet::Singular),
        map(multi_selection_set, SelectionSet::Multi),
    ))(input)
}

pub fn path_set(input: &str) -> IResult<&str, Expression<'_>> {
    terminated(map(singular_selection_set, Expression::SelectionSet), eof)(input)
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[test]
    fn invalid_path_set() {
        let raw = "foo::";
        assert!(path_set(raw).is_err());
    }

    #[rstest]
    #[case("a(b: \"foo\")", FieldArgValue::StringLiteral("foo".to_string()))]
    #[case("a(b: 10)", FieldArgValue::IntegerLiteral(10))]
    #[case("a(b: true)", FieldArgValue::BoolLiteral(true))]
    #[case("a(b: false)", FieldArgValue::BoolLiteral(false))]
    #[case("a(b: C)", FieldArgValue::Enum("C".to_string()))]
    #[case("a(b: FOO_BAR)", FieldArgValue::Enum("FOO_BAR".to_string()))]
    #[case("a(b: *)", FieldArgValue::Wildcard)]
    fn single_argument(#[case] raw: &str, #[case] expected_field_arg_value: FieldArgValue) {
        let (_, expr) = path_set(raw).expect("parse should not fail");
        let Expression::SelectionSet(selection) = expr;
        let SingularSelectionSet::Explicit(field, _) = selection else { panic!("selection should be explicit") };

        let Field { name, args } = field;
        assert_eq!(name, "a");

        let args = args.expect("args must not be None");
        let first_arg = args.first().expect("must have one argument");
        assert_eq!(first_arg.name, "b");
        assert_eq!(first_arg.value, expected_field_arg_value);
    }

    #[test]
    fn multiple_arguments() {
        let raw = "a(b: 10, c: true)";
        let (_, expr) = path_set(raw).expect("parse should not fail");
        let Expression::SelectionSet(selection) = expr;
        let SingularSelectionSet::Explicit(field, _) = selection else { panic!("selection should be explicit") };

        let Field { name, args } = field;
        assert_eq!(name, "a");

        let args = args.expect("args must not be None");

        let first_arg = args.get(0).expect("must have two arguments");
        assert_eq!(first_arg.name, "b");
        assert_eq!(first_arg.value, FieldArgValue::IntegerLiteral(10));

        let second_arg = args.get(1).expect("must have two arguments");
        assert_eq!(second_arg.name, "c");
        assert_eq!(second_arg.value, FieldArgValue::BoolLiteral(true));
    }
}
