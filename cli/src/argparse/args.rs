//! Parsers for argument lists -- arrays of strings
use super::ArgList;
use failure::format_err;
use nom::bytes::complete::tag as nomtag;
use nom::{
    branch::*,
    bytes::complete::is_a,
    character::complete::*,
    combinator::*,
    error::{Error, ErrorKind},
    multi::*,
    sequence::*,
    Err, IResult,
};

/// Recognizes any argument
pub(super) fn any(input: &str) -> IResult<&str, &str> {
    rest(input)
}

/// Recognizes a literal string
pub(super) fn literal(literal: &'static str) -> impl Fn(&str) -> IResult<&str, &str> {
    move |input: &str| all_consuming(nomtag(literal))(input)
}

/// Recognizes a comma-separated list of ID's (integers or UUID prefixes)
pub(super) fn id_list(input: &str) -> IResult<&str, Vec<&str>> {
    fn hex_n(n: usize) -> impl Fn(&str) -> IResult<&str, &str> {
        move |input: &str| recognize(many_m_n(n, n, is_a(&b"0123456789abcdefABCDEF"[..])))(input)
    }
    separated_list1(
        char(','),
        alt((
            digit1,
            hex_n(8),
            recognize(tuple((hex_n(8), char('-'), hex_n(4)))),
            recognize(tuple((hex_n(8), char('-'), hex_n(4), char('-'), hex_n(4)))),
            recognize(tuple((
                hex_n(8),
                char('-'),
                hex_n(4),
                char('-'),
                hex_n(4),
                char('-'),
                hex_n(4),
            ))),
            recognize(tuple((
                hex_n(8),
                char('-'),
                hex_n(4),
                char('-'),
                hex_n(4),
                char('-'),
                hex_n(4),
                char('-'),
                hex_n(12),
            ))),
        )),
    )(input)
}

/// Recognizes a tag prefixed with `+` and returns the tag value
pub(super) fn plus_tag(input: &str) -> IResult<&str, &str> {
    fn to_tag<'a>(input: (char, &'a str)) -> Result<&'a str, ()> {
        Ok(input.1)
    }
    map_res(
        all_consuming(tuple((char('+'), recognize(pair(alpha1, alphanumeric0))))),
        to_tag,
    )(input)
}

/// Recognizes a tag prefixed with `-` and returns the tag value
pub(super) fn minus_tag(input: &str) -> IResult<&str, &str> {
    fn to_tag<'a>(input: (char, &'a str)) -> Result<&'a str, ()> {
        Ok(input.1)
    }
    map_res(
        all_consuming(tuple((char('-'), recognize(pair(alpha1, alphanumeric0))))),
        to_tag,
    )(input)
}

/// Recognizes a tag prefixed with either `-` or `+`, returning true for + and false for -
pub(super) fn tag(input: &str) -> IResult<&str, (bool, &str)> {
    fn to_plus<'a>(input: &'a str) -> Result<(bool, &'a str), ()> {
        Ok((true, input))
    }
    fn to_minus<'a>(input: &'a str) -> Result<(bool, &'a str), ()> {
        Ok((false, input))
    }
    alt((map_res(plus_tag, to_plus), map_res(minus_tag, to_minus)))(input)
}

/// Consume a single argument from an argument list that matches the given string parser (one
/// of the other functions in this module).  The given parser must consume the entire input.
pub(super) fn arg_matching<'a, O, F>(f: F) -> impl Fn(ArgList<'a>) -> IResult<ArgList, O>
where
    F: Fn(&'a str) -> IResult<&'a str, O>,
{
    move |input: ArgList<'a>| {
        if let Some(arg) = input.get(0) {
            return match f(arg) {
                Ok(("", rv)) => Ok((&input[1..], rv)),
                // single-arg parsres must consume the entire arg
                Ok((_, _)) => unreachable!(),
                // single-arg parsers are all complete parsers
                Err(Err::Incomplete(_)) => unreachable!(),
                // for error and failure, rewrite to an error at this position in the arugment list
                Err(Err::Error(Error { input: _, code })) => Err(Err::Error(Error { input, code })),
                Err(Err::Failure(Error { input: _, code })) => {
                    Err(Err::Failure(Error { input, code }))
                }
            };
        }

        Err(Err::Error(Error {
            input,
            // since we're using nom's built-in Error, our choices here are limited, but tihs
            // occurs when there's no argument where one is expected, so Eof seems appropriate
            code: ErrorKind::Eof,
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// create a &[&str] from vec notation
    macro_rules! argv {
	() => (
	    &[][..]
	);
	($($x:expr),* $(,)?) => (
	    &[$($x),*][..]
	);
    }

    #[test]
    fn test_arg_matching() {
        assert_eq!(
            arg_matching(tag)(argv!["+foo", "bar"]).unwrap(),
            (argv!["bar"], (true, "foo"))
        );
        assert_eq!(
            arg_matching(tag)(argv!["-foo", "bar"]).unwrap(),
            (argv!["bar"], (false, "foo"))
        );
        assert!(arg_matching(tag)(argv!["foo", "bar"]).is_err());
    }

    #[test]
    fn test_plus_tag() {
        assert_eq!(plus_tag("+abc").unwrap().1, "abc");
        assert_eq!(plus_tag("+abc123").unwrap().1, "abc123");
        assert!(plus_tag("-abc123").is_err());
        assert!(plus_tag("+abc123  ").is_err());
        assert!(plus_tag("  +abc123").is_err());
        assert!(plus_tag("+1abc").is_err());
    }

    #[test]
    fn test_minus_tag() {
        assert_eq!(minus_tag("-abc").unwrap().1, "abc");
        assert_eq!(minus_tag("-abc123").unwrap().1, "abc123");
        assert!(minus_tag("+abc123").is_err());
        assert!(minus_tag("-abc123  ").is_err());
        assert!(minus_tag("  -abc123").is_err());
        assert!(minus_tag("-1abc").is_err());
    }

    #[test]
    fn test_tag() {
        assert_eq!(tag("-abc").unwrap().1, (false, "abc"));
        assert_eq!(tag("+abc123").unwrap().1, (true, "abc123"));
        assert!(tag("+abc123 --").is_err());
        assert!(tag("-abc123  ").is_err());
        assert!(tag("  -abc123").is_err());
        assert!(tag("-1abc").is_err());
    }

    #[test]
    fn test_literal() {
        assert_eq!(literal("list")("list").unwrap().1, "list");
        assert!(literal("list")("listicle").is_err());
        assert!(literal("list")(" list ").is_err());
        assert!(literal("list")("LiSt").is_err());
        assert!(literal("list")("denylist").is_err());
    }
}
