use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::{char, multispace1};
use nom::combinator::{map, value, verify};
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded};
use nom::IResult;

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

/// Parse an escaped character: \n, \t, \r etc.
fn parse_escaped_char(input: &str) -> IResult<&str, char> {
    preceded(
        char('\\'),
        alt((
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(input)
}

/// Parse a backslash, followed by any amount of whitespace. This is used later
/// to discard any escaped whitespace.
fn parse_escaped_whitespace<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    preceded(char('\\'), multispace1)(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    // `is_not` parses a string of 0 or more characters that aren't one of the
    // given characters.
    let not_quote_slash = is_not("\"\\");

    // `verify` runs a parser, then runs a verification function on the output of
    // the parser. The verification function accepts out output only if it
    // returns true. In this case, we want to ensure that the output of is_not
    // is non-empty.
    verify(not_quote_slash, |s: &str| !s.is_empty())(input)
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a>(input: &'a str) -> IResult<&'a str, StringFragment<'a>> {
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))(input)
}

/// Parse a string.
pub fn string_literal(input: &str) -> IResult<&str, String> {
    let build_string = fold_many0(parse_fragment, String::new, |mut string, fragment| {
        match fragment {
            StringFragment::Literal(s) => string.push_str(s),
            StringFragment::EscapedChar(c) => string.push(c),
            StringFragment::EscapedWS => {}
        }
        string
    });

    delimited(char('"'), build_string, char('"'))(input)
}
