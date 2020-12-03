use super::args::{any, arg_matching, literal};
use super::ArgList;
use nom::{
    branch::*, character::complete::*, combinator::*, multi::fold_many0, sequence::*, Err, IResult,
};
use taskchampion::Status;

/// A modification represents a change to a task: adding or removing tags, setting the
/// description, and so on.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Modification {
    pub description: Option<String>,
    pub status: Option<Status>,
}

/// A single argument that is part of a modification, used internally to this module
enum ModArg<'a> {
    Description(&'a str),
    Done,
}

impl Modification {
    pub(super) fn parse(input: ArgList) -> IResult<ArgList, Modification> {
        fn fold(mut acc: Modification, mod_arg: ModArg) -> Modification {
            match mod_arg {
                ModArg::Description(description) => {
                    if let Some(existing) = acc.description {
                        acc.description = Some(format!("{} {}", existing, description));
                    } else {
                        acc.description = Some(description.to_string());
                    }
                }
                ModArg::Done => {
                    acc.status = Some(Status::Completed);
                }
            }
            acc
        }
        fold_many0(
            alt((Self::done, Self::description)),
            Modification {
                ..Default::default()
            },
            fold,
        )(input)
    }

    fn description(input: ArgList) -> IResult<ArgList, ModArg> {
        fn to_modarg(input: &str) -> Result<ModArg, ()> {
            Ok(ModArg::Description(input))
        }
        map_res(arg_matching(any), to_modarg)(input)
    }

    // temporary
    fn done(input: ArgList) -> IResult<ArgList, ModArg> {
        fn to_modarg(_: &str) -> Result<ModArg, ()> {
            Ok(ModArg::Done)
        }
        map_res(arg_matching(literal("done")), to_modarg)(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO: import this from crate::argparse::argv
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
    fn test_empty() {
        let (input, modification) = Modification::parse(argv![]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_single_arg_description() {
        let (input, modification) = Modification::parse(argv!["newdesc"]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                description: Some("newdesc".to_owned()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_multi_arg_description() {
        let (input, modification) = Modification::parse(argv!["new", "desc", "fun"]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                description: Some("new desc fun".to_owned()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_multi_arg_description_with_done() {
        let (input, modification) =
            Modification::parse(argv!["new", "desc", "done", "fun"]).unwrap();
        assert_eq!(input.len(), 0);
        assert_eq!(
            modification,
            Modification {
                description: Some("new desc fun".to_owned()),
                status: Some(Status::Completed),
                ..Default::default()
            }
        );
    }
}
