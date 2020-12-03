#![allow(dead_code)]
#![allow(unused_imports)]

type ArgList<'a> = &'a [&'a str];

mod args;
mod command;
mod filter;
mod modification;
mod report;
mod subcommand;

use command::Command;
use filter::Filter;
use modification::Modification;
use report::Report;
use subcommand::Subcommand;

#[cfg(test)]
mod test;
