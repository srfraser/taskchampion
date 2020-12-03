use super::args::*;
use super::{ArgList, Filter};
use failure::{format_err, Fallible};
use nom::error::ErrorKind;
use nom::{
    branch::*, character::complete::*, combinator::*, multi::fold_many0, sequence::*, Err, IResult,
};

/// A report specifies a filter as well as a sort order and information about which
/// task attributes to display
#[derive(Debug, PartialEq)]
pub(crate) struct Report {
    pub filter: Option<Filter>,
}

impl Report {
    pub(super) fn parse(input: ArgList) -> IResult<ArgList, Report> {
        let (input, filter) = Filter::parse(input)?;
        Ok((
            input,
            Report {
                filter: Some(filter),
            },
        ))
    }
}
