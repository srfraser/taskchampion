use super::args::*;
use super::{ArgList, Subcommand};
use failure::{format_err, Fallible};
use nom::error::ErrorKind;
use nom::{
    branch::*, character::complete::*, combinator::*, multi::fold_many0, sequence::*, Err, IResult,
};

/// A command is the overall command that the CLI should execute.
#[derive(Debug, PartialEq)]
pub struct Command {
    pub(crate) command_name: String,
    pub(crate) subcommand: Subcommand,
}

impl Command {
    pub(super) fn parse(input: ArgList) -> IResult<ArgList, Command> {
        fn to_command(input: (&str, Subcommand)) -> Result<Command, ()> {
            let command = Command {
                command_name: input.0.to_owned(),
                subcommand: input.1,
            };
            Ok(command)
        }
        map_res(
            all_consuming(tuple((arg_matching(any), Subcommand::parse))),
            to_command,
        )(input)
    }

    /// Parse a command from the given list of strings.
    pub fn from_argv(input: &[String]) -> Fallible<Command> {
        let arglist: Vec<&str> = input.iter().map(|s| s.as_ref()).collect();
        match Command::parse(&arglist[..]) {
            Ok((&[], cmd)) => Ok(cmd),
            Ok((trailing, _)) => Err(format_err!(
                "command line has trailing arguments: {:?}",
                trailing
            )),
            Err(Err::Incomplete(_)) => unreachable!(),
            Err(Err::Error(e)) => Err(format_err!("command line not recognized: {:?}", e)),
            Err(Err::Failure(e)) => Err(format_err!("command line not recognized: {:?}", e)),
        }
    }
}
