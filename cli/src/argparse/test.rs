use super::*;

/// create a &[String] from vec notation
macro_rules! owned_argv {
    () => (
        &[;String][..]
    );
    ($($x:expr),* $(,)?) => (
        &[$(String::from($x)),*][..]
    );
}

/// Shorthand for a Command instance
macro_rules! cmd {
    ($subcommand:expr) => {
        Command {
            command_name: "task".to_owned(),
            subcommand: $subcommand,
        }
    };
}

#[test]
fn test_add_description() {
    let subcommand = Subcommand::Add(Modification {
        description: Some("foo".to_owned()),
        ..Default::default()
    });
    assert_eq!(
        Command::from_argv(owned_argv!["task", "add", "foo"]).unwrap(),
        cmd!(subcommand)
    );
}

#[test]
fn test_add_description_multi_arg() {
    let subcommand = Subcommand::Add(Modification {
        description: Some("foo bar bing".to_owned()),
        ..Default::default()
    });
    assert_eq!(
        Command::from_argv(owned_argv!["task", "add", "foo", "bar", "bing"]).unwrap(),
        cmd!(subcommand)
    );
}

#[test]
fn test_gc() {
    assert_eq!(
        Command::from_argv(owned_argv!["task", "gc"]).unwrap(),
        cmd!(Subcommand::Gc)
    );
}

#[test]
fn test_gc_extra_args() {
    assert!(Command::from_argv(owned_argv!["task", "gc", "uhh"]).is_err());
}

#[test]
fn test_sync() {
    assert_eq!(
        Command::from_argv(owned_argv!["task", "sync"]).unwrap(),
        cmd!(Subcommand::Sync)
    );
}
