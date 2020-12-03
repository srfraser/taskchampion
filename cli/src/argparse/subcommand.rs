use super::args::*;
use super::{ArgList, Filter, Modification, Report};
use failure::{format_err, Fallible};
use nom::error::ErrorKind;
use nom::{
    branch::*, character::complete::*, combinator::*, multi::fold_many0, sequence::*, Err, IResult,
};

/// A subcommand is the specific operation that the CLI should execute.
#[derive(Debug, PartialEq)]
pub(crate) enum Subcommand {
    /// Add a new task
    Add(Modification),

    /// Modify 0 or more existing tasks
    Modify {
        filter: Filter,
        modification: Modification,
    },

    /// Lists (reports)
    List(Report),

    /// Per-task information (typically one task)
    Info {
        filter: Filter,
        debug: bool,
    },

    /// Basic operations without args
    Gc,
    Sync,
}

impl Subcommand {
    pub(super) fn parse<'a>(input: ArgList) -> IResult<ArgList, Subcommand> {
        alt((
            Self::add,
            Self::modify,
            Self::list,
            Self::info,
            Self::gc,
            Self::sync,
        ))(input)
    }

    fn add(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(input: (&str, Modification)) -> Result<Subcommand, ()> {
            Ok(Subcommand::Add(input.1))
        }
        map_res(
            pair(arg_matching(literal("add")), Modification::parse),
            to_subcommand,
        )(input)
    }

    fn modify(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(input: (Filter, &str, Modification)) -> Result<Subcommand, ()> {
            Ok(Subcommand::Modify {
                filter: input.0,
                modification: input.2,
            })
        }
        map_res(
            tuple((
                Filter::parse,
                arg_matching(literal("modify")),
                Modification::parse,
            )),
            to_subcommand,
        )(input)
    }

    fn list(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(input: (Report, &str)) -> Result<Subcommand, ()> {
            Ok(Subcommand::List(input.0))
        }
        map_res(
            pair(Report::parse, arg_matching(literal("list"))),
            to_subcommand,
        )(input)
    }

    fn info(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(input: (Filter, &str)) -> Result<Subcommand, ()> {
            let debug = input.1 == "debug";
            Ok(Subcommand::Info {
                filter: input.0,
                debug,
            })
        }
        map_res(
            pair(
                Filter::parse,
                alt((
                    arg_matching(literal("info")),
                    arg_matching(literal("debug")),
                )),
            ),
            to_subcommand,
        )(input)
    }

    fn gc(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(_: &str) -> Result<Subcommand, ()> {
            Ok(Subcommand::Gc)
        }
        map_res(arg_matching(literal("gc")), to_subcommand)(input)
    }

    fn sync(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(_: &str) -> Result<Subcommand, ()> {
            Ok(Subcommand::Sync)
        }
        map_res(arg_matching(literal("sync")), to_subcommand)(input)
    }
}

// TODO: tests
