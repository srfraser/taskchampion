use super::args::{arg_matching, id_list};
use super::ArgList;
use failure::{format_err, Fallible};
use nom::error::ErrorKind;
use nom::{
    branch::*, character::complete::*, combinator::*, multi::fold_many0, sequence::*, Err, IResult,
};
use taskchampion::Uuid;

/// A filter selects a set of tasks
#[derive(Debug, PartialEq, Default, Clone)]
pub(crate) struct Filter {
    /// A list of numeric IDs or prefixes of UUIDs
    pub(crate) id_list: Option<Vec<String>>,
}

enum FilterArg {
    IdList(Vec<String>),
}

impl Filter {
    pub(super) fn parse(input: ArgList) -> IResult<ArgList, Filter> {
        fn fold(mut acc: Filter, mod_arg: FilterArg) -> Filter {
            match mod_arg {
                FilterArg::IdList(mut id_list) => {
                    if let Some(ref mut existing) = acc.id_list {
                        // given multiple ID lists, concatenate them to represent
                        // an "OR" between them.
                        existing.append(&mut id_list);
                    } else {
                        acc.id_list = Some(id_list);
                    }
                }
            }
            acc
        }
        fold_many0(
            Self::id_list,
            Filter {
                ..Default::default()
            },
            fold,
        )(input)
    }

    fn id_list(input: ArgList) -> IResult<ArgList, FilterArg> {
        fn to_filterarg(mut input: Vec<&str>) -> Result<FilterArg, ()> {
            Ok(FilterArg::IdList(
                input.drain(..).map(str::to_owned).collect(),
            ))
        }
        map_res(arg_matching(id_list), to_filterarg)(input)
    }
}

// TODO: tests
