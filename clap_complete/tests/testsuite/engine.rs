#![cfg(feature = "unstable-dynamic")]

use std::fs;
use std::path::Path;

use clap::{builder::PossibleValue, ArgGroup, Command};
use clap_complete::engine::{
    ArgValueCandidates, ArgValueCompleter, CompletionCandidate, PathCompleter, SubcommandCandidates,
};
use snapbox::assert_data_eq;

macro_rules! complete {
    ($cmd:expr, $input:expr$(, current_dir = $current_dir:expr)? $(,)?) => {
        {
            #[allow(unused)]
            let current_dir: Option<&Path> = None;
            $(let current_dir = $current_dir;)?
            complete(&mut $cmd, $input, current_dir)
        }
    }
}

#[test]
fn suggest_subcommand_subset() {
    let mut cmd = Command::new("exhaustive")
        .subcommand(Command::new("hello-world"))
        .subcommand(Command::new("hello-moon"))
        .subcommand(Command::new("goodbye-world"));

    assert_data_eq!(
        complete!(cmd, "he"),
        snapbox::str![[r#"
hello-world
hello-moon
help	Print this message or the help of the given subcommand(s)
"#]],
    );
}

#[test]
fn suggest_hidden_long_flags() {
    let mut cmd = Command::new("exhaustive")
        .arg(clap::Arg::new("hello-world-visible").long("hello-world-visible"))
        .arg(
            clap::Arg::new("hello-world-hidden")
                .long("hello-world-hidden")
                .hide(true),
        );

    assert_data_eq!(
        complete!(cmd, "--hello-world"),
        snapbox::str!["--hello-world-visible"]
    );

    assert_data_eq!(
        complete!(cmd, "--hello-world-h"),
        snapbox::str!["--hello-world-hidden"]
    );
}

#[test]
fn suggest_hidden_subcommand_and_aliases() {
    let mut cmd = Command::new("exhaustive")
        .subcommand(
            Command::new("test_visible")
                .visible_alias("test_visible-alias_visible")
                .alias("test_visible-alias_hidden"),
        )
        .subcommand(
            Command::new("test_hidden")
                .visible_alias("test_hidden-alias_visible")
                .alias("test_hidden-alias_hidden")
                .hide(true),
        );

    assert_data_eq!(
        complete!(cmd, "test"),
        snapbox::str![[r#"
test_visible
test_visible-alias_visible
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "test_h"),
        snapbox::str![[r#"
test_hidden
test_hidden-alias_visible
test_hidden-alias_hidden
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "test_hidden-alias_h"),
        snapbox::str!["test_hidden-alias_hidden"]
    );
}

#[test]
fn suggest_subcommand_aliases() {
    let mut cmd = Command::new("exhaustive")
        .subcommand(
            Command::new("hello-world")
                .visible_alias("hello-world-foo")
                .alias("hidden-world"),
        )
        .subcommand(
            Command::new("hello-moon")
                .visible_alias("hello-moon-foo")
                .alias("hidden-moon"),
        )
        .subcommand(
            Command::new("goodbye-world")
                .visible_alias("goodbye-world-foo")
                .alias("hidden-goodbye"),
        );

    assert_data_eq!(
        complete!(cmd, "hello"),
        snapbox::str![[r#"
hello-world
hello-world-foo
hello-moon
hello-moon-foo
"#]],
    );
}

#[test]
fn suggest_hidden_possible_value() {
    let mut cmd = Command::new("exhaustive").arg(
        clap::Arg::new("possible_value").long("test").value_parser([
            PossibleValue::new("test-visible").help("Say hello to the world"),
            PossibleValue::new("test-hidden")
                .help("Say hello to the moon")
                .hide(true),
        ]),
    );

    assert_data_eq!(
        complete!(cmd, "--test=test"),
        snapbox::str!["--test=test-visible	Say hello to the world"]
    );

    assert_data_eq!(
        complete!(cmd, "--test=test-h"),
        snapbox::str!["--test=test-hidden	Say hello to the moon"]
    );
}

#[test]
fn suggest_hidden_long_flag_aliases() {
    let mut cmd = Command::new("exhaustive")
        .arg(
            clap::Arg::new("test_visible")
                .long("test_visible")
                .visible_alias("test_visible-alias_visible")
                .alias("test_visible-alias_hidden"),
        )
        .arg(
            clap::Arg::new("test_hidden")
                .long("test_hidden")
                .visible_alias("test_hidden-alias_visible")
                .alias("test_hidden-alias_hidden")
                .hide(true),
        );

    assert_data_eq!(
        complete!(cmd, "--test"),
        snapbox::str![[r#"
--test_visible
--test_visible-alias_visible
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--test_h"),
        snapbox::str![[r#"
--test_hidden
--test_hidden-alias_visible
--test_hidden-alias_hidden
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--test_visible-alias_h"),
        snapbox::str!["--test_visible-alias_hidden"]
    );

    assert_data_eq!(
        complete!(cmd, "--test_hidden-alias_h"),
        snapbox::str!["--test_hidden-alias_hidden"]
    );
}

#[test]
fn suggest_long_flag_subset() {
    let mut cmd = Command::new("exhaustive")
        .arg(
            clap::Arg::new("hello-world")
                .long("hello-world")
                .action(clap::ArgAction::Count),
        )
        .arg(
            clap::Arg::new("hello-moon")
                .long("hello-moon")
                .action(clap::ArgAction::Count),
        )
        .arg(
            clap::Arg::new("goodbye-world")
                .long("goodbye-world")
                .action(clap::ArgAction::Count),
        );

    assert_data_eq!(
        complete!(cmd, "--he"),
        snapbox::str![[r#"
--hello-world
--hello-moon
--help	Print help
"#]],
    );
}

#[test]
fn suggest_possible_value_subset() {
    let name = "exhaustive";
    let mut cmd = Command::new(name).arg(clap::Arg::new("hello-world").value_parser([
        PossibleValue::new("hello-world").help("Say hello to the world"),
        "hello-moon".into(),
        "goodbye-world".into(),
    ]));

    assert_data_eq!(
        complete!(cmd, "hello"),
        snapbox::str![[r#"
hello-world	Say hello to the world
hello-moon
"#]],
    );
}

#[test]
fn suggest_additional_short_flags() {
    let mut cmd = Command::new("exhaustive")
        .arg(
            clap::Arg::new("a")
                .short('a')
                .action(clap::ArgAction::Count),
        )
        .arg(
            clap::Arg::new("b")
                .short('b')
                .action(clap::ArgAction::Count),
        )
        .arg(
            clap::Arg::new("c")
                .short('c')
                .action(clap::ArgAction::Count),
        );

    assert_data_eq!(
        complete!(cmd, "-a"),
        snapbox::str![[r#"
-aa
-ab
-ac
-ah	Print help
"#]],
    );
}

#[test]
fn suggest_subcommand_positional() {
    let mut cmd = Command::new("exhaustive").subcommand(Command::new("hello-world").arg(
        clap::Arg::new("hello-world").value_parser([
            PossibleValue::new("hello-world").help("Say hello to the world"),
            "hello-moon".into(),
            "goodbye-world".into(),
        ]),
    ));

    assert_data_eq!(
        complete!(cmd, "hello-world [TAB]"),
        snapbox::str![[r#"
hello-world	Say hello to the world
hello-moon
goodbye-world
--help	Print help (see more with '--help')
"#]],
    );
}

#[test]
fn suggest_subcommand_positional_after_escape() {
    let mut cmd = Command::new("exhaustive").subcommand(Command::new("hello-world").arg(
        clap::Arg::new("hello-world").value_parser([
            PossibleValue::new("hello-world").help("Say hello to the world"),
            "hello-moon".into(),
            "goodbye-world".into(),
        ]),
    ));

    assert_data_eq!(
        complete!(cmd, "hello-world -- [TAB]"),
        snapbox::str![[r#"
hello-world	Say hello to the world
hello-moon
goodbye-world
"#]],
    );
}

#[test]
fn suggest_multiple_positional_after_escape() {
    let mut cmd =
        Command::new("exhaustive").arg(clap::Arg::new("hello-world").num_args(0..).value_parser([
            PossibleValue::new("hello-world"),
            "hello-moon".into(),
            "goodbye-world".into(),
        ]));

    assert_data_eq!(
        complete!(cmd, "-- hello-moon [TAB]"),
        snapbox::str![[r#"
hello-world
hello-moon
goodbye-world
"#]],
    );
}

#[test]
fn suggest_argument_value() {
    let mut cmd = Command::new("dynamic")
        .arg(
            clap::Arg::new("format")
                .long("format")
                .short('F')
                .value_parser(["json", "yaml", "toml"]),
        )
        .arg(
            clap::Arg::new("stream")
                .long("stream")
                .short('S')
                .value_parser(["stdout", "stderr"]),
        )
        .arg(
            clap::Arg::new("count")
                .long("count")
                .short('c')
                .action(clap::ArgAction::Count),
        )
        .arg(clap::Arg::new("positional").value_parser(["pos_a", "pos_b", "pos_c"]))
        .args_conflicts_with_subcommands(true);

    assert_data_eq!(
        complete!(cmd, "--format [TAB]"),
        snapbox::str![[r#"
json
yaml
toml
"#]],
    );

    assert_data_eq!(
        complete!(cmd, "-F [TAB]"),
        snapbox::str![[r#"
json
yaml
toml
"#]],
    );

    assert_data_eq!(complete!(cmd, "--format j[TAB]"), snapbox::str!["json"],);

    assert_data_eq!(complete!(cmd, "-F j[TAB]"), snapbox::str!["json"],);

    assert_data_eq!(complete!(cmd, "--format t[TAB]"), snapbox::str!["toml"],);

    assert_data_eq!(complete!(cmd, "-F t[TAB]"), snapbox::str!["toml"],);

    assert_data_eq!(
        complete!(cmd, "-cccF [TAB]"),
        snapbox::str![[r#"
json
yaml
toml
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--format toml [TAB]"),
        snapbox::str![[r#"
pos_a
pos_b
pos_c
--format
--stream
--count
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-cS[TAB]"),
        snapbox::str![[r#"
-cSstdout
-cSstderr
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-cS=[TAB]"),
        snapbox::str![[r#"
-cS=stdout
-cS=stderr
"#]]
    );

    assert_data_eq!(complete!(cmd, "-cS=stdo[TAB]"), snapbox::str!["-cS=stdout"]);

    assert_data_eq!(complete!(cmd, "-cSF[TAB]"), snapbox::str![]);

    assert_data_eq!(complete!(cmd, "-cSF=[TAB]"), snapbox::str![]);
}

#[test]
fn suggest_argument_multi_values() {
    let mut cmd = Command::new("dynamic")
        .arg(
            clap::Arg::new("certain-num")
                .long("certain-num")
                .short('Y')
                .value_parser(["val1", "val2", "val3"])
                .num_args(3),
        )
        .arg(
            clap::Arg::new("uncertain-num")
                .long("uncertain-num")
                .short('N')
                .value_parser(["val1", "val2", "val3"])
                .num_args(1..=3),
        );

    assert_data_eq!(
        complete!(cmd, "--certain-num [TAB]"),
        snapbox::str![[r#"
val1
val2
val3
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--certain-num val1 [TAB]"),
        snapbox::str![[r#"
val1
val2
val3
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--certain-num val1 val2 val3 [TAB]"),
        snapbox::str![[r#"
--certain-num
--uncertain-num
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--uncertain-num [TAB]"),
        snapbox::str![[r#"
val1
val2
val3
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--uncertain-num val1 [TAB]"),
        snapbox::str![[r#"
val1
val2
val3
"#]]
    );

    // After one value the minimum is met, so a flag-like word switches to
    // another option, while plain words keep completing option values.
    assert_data_eq!(
        complete!(cmd, "--uncertain-num val1 --[TAB]"),
        snapbox::str![[r#"
--certain-num
--uncertain-num
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--uncertain-num val1 val2 val3 [TAB]"),
        snapbox::str![[r#"
--certain-num
--uncertain-num
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-Y [TAB]"),
        snapbox::str![[r#"
val1
val2
val3
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-Y val1 [TAB]"),
        snapbox::str![[r#"
val1
val2
val3
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-Y val1 val2 val3 [TAB]"),
        snapbox::str![[r#"
--certain-num
--uncertain-num
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-N [TAB]"),
        snapbox::str![[r#"
val1
val2
val3
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-N val1 [TAB]"),
        snapbox::str![[r#"
val1
val2
val3
"#]]
    );

    // A flag-like word after the minimum switches to another option.
    assert_data_eq!(
        complete!(cmd, "-N val1 --[TAB]"),
        snapbox::str![[r#"
--certain-num
--uncertain-num
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-N val1 val2 val3 [TAB]"),
        snapbox::str![[r#"
--certain-num
--uncertain-num
--help	Print help
"#]]
    );
}

#[test]
fn suggest_value_hint_file_path() {
    let mut cmd = Command::new("dynamic")
        .arg(
            clap::Arg::new("input")
                .long("input")
                .short('i')
                .value_hint(clap::ValueHint::FilePath),
        )
        .args_conflicts_with_subcommands(true);

    let testdir = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let testdir_path = testdir.path().unwrap();

    fs::write(testdir_path.join("a_file"), "").unwrap();
    fs::write(testdir_path.join(".a_file"), "").unwrap();
    fs::write(testdir_path.join("b_file"), "").unwrap();
    fs::write(testdir_path.join(".b_file"), "").unwrap();
    fs::create_dir_all(testdir_path.join("c_dir")).unwrap();
    fs::create_dir_all(testdir_path.join(".c_dir")).unwrap();
    fs::create_dir_all(testdir_path.join("d_dir")).unwrap();
    fs::create_dir_all(testdir_path.join(".d_dir")).unwrap();

    assert_data_eq!(
        complete!(cmd, "--input [TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
a_file
b_file
c_dir/
d_dir/
"#]],
    );

    assert_data_eq!(
        complete!(cmd, "--input a[TAB]", current_dir = Some(testdir_path)),
        snapbox::str!["a_file"],
    );
    assert_data_eq!(
        complete!(cmd, "--input .[TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
./a_file
./b_file
./c_dir/
./d_dir/
"#]],
    );
    assert_data_eq!(
        complete!(cmd, "--input .a[TAB]", current_dir = Some(testdir_path)),
        snapbox::str![".a_file"],
    );
}

#[test]
fn suggest_value_path_file() {
    let testdir = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let testdir_path = testdir.path().unwrap();

    fs::write(testdir_path.join("a_file"), "").unwrap();
    fs::write(testdir_path.join(".a_file"), "").unwrap();
    fs::write(testdir_path.join("b_file"), "").unwrap();
    fs::write(testdir_path.join(".b_file"), "").unwrap();
    fs::create_dir_all(testdir_path.join("c_dir")).unwrap();
    fs::create_dir_all(testdir_path.join(".c_dir")).unwrap();
    fs::create_dir_all(testdir_path.join("d_dir")).unwrap();
    fs::create_dir_all(testdir_path.join(".d_dir")).unwrap();

    let mut cmd = Command::new("dynamic")
        .arg(
            clap::Arg::new("input")
                .long("input")
                .short('i')
                .add(ArgValueCompleter::new(
                    PathCompleter::file()
                        .stdio()
                        .current_dir(testdir_path.to_owned()),
                )),
        )
        .args_conflicts_with_subcommands(true);

    assert_data_eq!(
        complete!(cmd, "--input [TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
a_file
b_file
c_dir/
d_dir/
-	stdio
"#]],
    );

    assert_data_eq!(
        complete!(cmd, "--input a[TAB]", current_dir = Some(testdir_path)),
        snapbox::str!["a_file"],
    );
    assert_data_eq!(
        complete!(cmd, "--input .[TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
./a_file
./b_file
./c_dir/
./d_dir/
"#]],
    );
    assert_data_eq!(
        complete!(cmd, "--input .a[TAB]", current_dir = Some(testdir_path)),
        snapbox::str![".a_file"],
    );
}

#[test]
fn suggest_value_path_dir() {
    let testdir = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let testdir_path = testdir.path().unwrap();

    fs::write(testdir_path.join("a_file"), "").unwrap();
    fs::write(testdir_path.join(".a_file"), "").unwrap();
    fs::write(testdir_path.join("b_file"), "").unwrap();
    fs::write(testdir_path.join(".b_file"), "").unwrap();
    fs::create_dir_all(testdir_path.join("c_dir")).unwrap();
    fs::create_dir_all(testdir_path.join(".c_dir")).unwrap();
    fs::create_dir_all(testdir_path.join("d_dir")).unwrap();
    fs::create_dir_all(testdir_path.join(".d_dir")).unwrap();

    let mut cmd = Command::new("dynamic")
        .arg(
            clap::Arg::new("input")
                .long("input")
                .short('i')
                .add(ArgValueCompleter::new(
                    PathCompleter::dir().current_dir(testdir_path.to_owned()),
                )),
        )
        .args_conflicts_with_subcommands(true);

    assert_data_eq!(
        complete!(cmd, "--input [TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
.
c_dir/
d_dir/
"#]],
    );

    assert_data_eq!(
        complete!(cmd, "--input c[TAB]", current_dir = Some(testdir_path)),
        snapbox::str!["c_dir/"],
    );
    assert_data_eq!(
        complete!(cmd, "--input .[TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
./c_dir/
./d_dir/
"#]],
    );
    assert_data_eq!(
        complete!(cmd, "--input .c[TAB]", current_dir = Some(testdir_path)),
        snapbox::str![".c_dir/"],
    );
}

#[cfg(unix)]
#[test]
fn suggest_value_hint_file_path_symlink_to_dir() {
    use std::os::unix::fs::symlink;

    let mut cmd = Command::new("dynamic").arg(
        clap::Arg::new("input")
            .long("input")
            .short('i')
            .value_hint(clap::ValueHint::FilePath),
    );

    let testdir = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let testdir_path = testdir.path().unwrap();

    fs::create_dir_all(testdir_path.join("real_dir")).unwrap();
    fs::write(testdir_path.join("real_dir/file.txt"), "").unwrap();
    symlink("real_dir", testdir_path.join("link_dir")).unwrap();

    // Symlink to directory should appear with trailing slash
    assert_data_eq!(
        complete!(cmd, "--input [TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
link_dir/
real_dir/
"#]],
    );

    // Should be able to complete through the symlink
    assert_data_eq!(
        complete!(cmd, "--input link_dir/[TAB]", current_dir = Some(testdir_path)),
        snapbox::str!["link_dir/file.txt"],
    );
}

#[cfg(unix)]
#[test]
fn suggest_value_hint_file_path_symlink_to_file() {
    use std::os::unix::fs::symlink;

    let mut cmd = Command::new("dynamic").arg(
        clap::Arg::new("input")
            .long("input")
            .short('i')
            .value_hint(clap::ValueHint::FilePath),
    );

    let testdir = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let testdir_path = testdir.path().unwrap();

    fs::write(testdir_path.join("real_file.txt"), "").unwrap();
    symlink("real_file.txt", testdir_path.join("link_file.txt")).unwrap();

    // Symlink to file should appear without trailing slash
    assert_data_eq!(
        complete!(cmd, "--input [TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
link_file.txt
real_file.txt
"#]],
    );
}

#[cfg(unix)]
#[test]
fn suggest_value_hint_dir_path_symlink() {
    use std::os::unix::fs::symlink;

    let mut cmd = Command::new("dynamic").arg(
        clap::Arg::new("input")
            .long("input")
            .short('i')
            .value_hint(clap::ValueHint::DirPath),
    );

    let testdir = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let testdir_path = testdir.path().unwrap();

    fs::create_dir_all(testdir_path.join("real_dir")).unwrap();
    fs::write(testdir_path.join("real_file.txt"), "").unwrap();
    symlink("real_dir", testdir_path.join("link_dir")).unwrap();
    symlink("real_file.txt", testdir_path.join("link_file.txt")).unwrap();

    // Symlink to directory should have trailing slash
    assert_data_eq!(
        complete!(cmd, "--input [TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
.
link_dir/
real_dir/
"#]],
    );
}

#[cfg(unix)]
#[test]
fn suggest_value_hint_file_path_broken_symlink() {
    use std::os::unix::fs::symlink;

    let mut cmd = Command::new("dynamic").arg(
        clap::Arg::new("input")
            .long("input")
            .short('i')
            .value_hint(clap::ValueHint::FilePath),
    );

    let testdir = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let testdir_path = testdir.path().unwrap();

    fs::write(testdir_path.join("real_file.txt"), "").unwrap();
    symlink("nonexistent", testdir_path.join("broken_link")).unwrap();

    // Broken symlink should not appear for FilePath (target doesn't exist)
    // but should not cause a crash
    assert_data_eq!(
        complete!(cmd, "--input [TAB]", current_dir = Some(testdir_path)),
        snapbox::str!["real_file.txt"],
    );
}

#[cfg(unix)]
#[test]
fn suggest_value_hint_any_path_broken_symlink() {
    use std::os::unix::fs::symlink;

    let mut cmd = Command::new("dynamic").arg(
        clap::Arg::new("input")
            .long("input")
            .short('i')
            .value_hint(clap::ValueHint::AnyPath),
    );

    let testdir = snapbox::dir::DirRoot::mutable_temp().unwrap();
    let testdir_path = testdir.path().unwrap();

    fs::write(testdir_path.join("real_file.txt"), "").unwrap();
    symlink("nonexistent", testdir_path.join("broken_link")).unwrap();

    // Broken symlink should appear for AnyPath since filter is |_| true
    assert_data_eq!(
        complete!(cmd, "--input [TAB]", current_dir = Some(testdir_path)),
        snapbox::str![[r#"
.
broken_link
real_file.txt
"#]],
    );
}

#[test]
fn suggest_custom_arg_value() {
    fn custom_completer() -> Vec<CompletionCandidate> {
        vec![
            CompletionCandidate::new("foo"),
            CompletionCandidate::new("bar"),
            CompletionCandidate::new("baz"),
        ]
    }

    let mut cmd = Command::new("dynamic").arg(
        clap::Arg::new("custom")
            .long("custom")
            .add(ArgValueCandidates::new(custom_completer)),
    );

    assert_data_eq!(
        complete!(cmd, "--custom [TAB]"),
        snapbox::str![[r#"
foo
bar
baz
"#]],
    );

    assert_data_eq!(
        complete!(cmd, "--custom b[TAB]"),
        snapbox::str![[r#"
bar
baz
"#]],
    );
}

#[test]
fn suggest_custom_arg_completer() {
    fn custom_completer(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
        let mut completions = vec![];
        let Some(current) = current.to_str() else {
            return completions;
        };

        if "foo".starts_with(current) {
            completions.push(CompletionCandidate::new("foo"));
        }
        if "bar".starts_with(current) {
            completions.push(CompletionCandidate::new("bar"));
        }
        if "baz".starts_with(current) {
            completions.push(CompletionCandidate::new("baz"));
        }
        completions
    }

    let mut cmd = Command::new("dynamic").arg(
        clap::Arg::new("custom")
            .long("custom")
            .add(ArgValueCompleter::new(custom_completer)),
    );

    assert_data_eq!(
        complete!(cmd, "--custom [TAB]"),
        snapbox::str![[r#"
foo
bar
baz
"#]]
    );
    assert_data_eq!(
        complete!(cmd, "--custom b[TAB]"),
        snapbox::str![[r#"
bar
baz
"#]]
    );
}

#[test]
fn suggest_custom_arg_completer_at_index() {
    struct UpstreamCompleter;

    impl clap_complete::engine::ValueCompleter for UpstreamCompleter {
        fn complete(&self, _current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
            // Falls back when callers use the index-unaware path.
            vec![CompletionCandidate::new("unindexed")]
        }

        fn complete_at(
            &self,
            arg_index: usize,
            current: &std::ffi::OsStr,
        ) -> Vec<CompletionCandidate> {
            let prefix = current.to_str().unwrap_or("");
            match arg_index {
                0 => ["origin", "upstream"]
                    .into_iter()
                    .filter(|name| name.starts_with(prefix))
                    .map(CompletionCandidate::new)
                    .collect(),
                1 => ["main", "master", "dev"]
                    .into_iter()
                    .filter(|name| name.starts_with(prefix))
                    .map(CompletionCandidate::new)
                    .collect(),
                _ => Vec::new(),
            }
        }
    }

    let mut cmd = Command::new("dynamic").arg(
        clap::Arg::new("set-upstream")
            .long("set-upstream")
            .short('u')
            .num_args(2)
            .value_names(["REMOTE", "BRANCH"])
            .add(ArgValueCompleter::new(UpstreamCompleter)),
    );

    assert_data_eq!(
        complete!(cmd, "--set-upstream [TAB]"),
        snapbox::str![[r#"
origin
upstream
"#]]
    );
    assert_data_eq!(
        complete!(cmd, "--set-upstream o[TAB]"),
        snapbox::str!["origin"]
    );
    assert_data_eq!(
        complete!(cmd, "--set-upstream origin [TAB]"),
        snapbox::str![[r#"
main
master
dev
"#]]
    );
    assert_data_eq!(
        complete!(cmd, "--set-upstream origin m[TAB]"),
        snapbox::str![[r#"
main
master
"#]]
    );
}

#[test]
fn suggest_multi_positional() {
    let mut cmd = Command::new("dynamic")
        .arg(clap::Arg::new("positional-1").value_parser(["pos_1_a", "pos_1_b", "pos_1_c"]))
        .arg(
            clap::Arg::new("positional-2")
                .value_parser(["pos_2_a", "pos_2_b", "pos_2_c"])
                .num_args(3),
        )
        .arg(
            clap::Arg::new("--format")
                .long("format")
                .short('F')
                .value_parser(["json", "yaml", "toml"]),
        );

    assert_data_eq!(
        complete!(cmd, "pos_1_a pos_2_a [TAB]"),
        snapbox::str![[r#"
pos_2_a
pos_2_b
pos_2_c
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "pos_1_a pos_2_a pos_2_b [TAB]"),
        snapbox::str![[r#"
pos_2_a
pos_2_b
pos_2_c
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--format json pos_1_a [TAB]"),
        snapbox::str![[r#"
pos_2_a
pos_2_b
pos_2_c
--format
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--format json pos_1_a pos_2_a [TAB]"),
        snapbox::str![[r#"
pos_2_a
pos_2_b
pos_2_c
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--format json pos_1_a pos_2_a pos_2_b pos_2_c [TAB]"),
        snapbox::str![[r#"
--format
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--format json -- pos_1_a pos_2_a [TAB]"),
        snapbox::str![[r#"
pos_2_a
pos_2_b
pos_2_c
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--format json -- pos_1_a pos_2_a pos_2_b [TAB]"),
        snapbox::str![[r#"
pos_2_a
pos_2_b
pos_2_c
"#]]
    );

    assert_data_eq!(
        complete!(
            cmd,
            "--format json -- pos_1_a pos_2_a pos_2_b pos_2_c [TAB]"
        ),
        snapbox::str![]
    );
}

#[test]
fn suggest_multi_positional_unbounded() {
    let mut cmd = Command::new("dynamic")
        .arg(
            clap::Arg::new("positional-1")
                .value_parser(["pos_1_a", "pos_1_b", "pos_1_c"])
                .num_args(2..),
        )
        .arg(
            clap::Arg::new("--format")
                .long("format")
                .short('F')
                .value_parser(["json", "yaml", "toml"]),
        );

    assert_data_eq!(
        complete!(cmd, "pos_1_a [TAB]"),
        snapbox::str![[r#"
pos_1_a
pos_1_b
pos_1_c
"#]]
    );
    assert_data_eq!(complete!(cmd, "pos_1_a --[TAB]"), snapbox::str![""]);
    assert_data_eq!(
        complete!(cmd, "pos_1_a --format [TAB]"),
        snapbox::str![[r#"
json
yaml
toml
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "pos_1_a --format json [TAB]"),
        snapbox::str![[r#"
pos_1_a
pos_1_b
pos_1_c
--format
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "pos_1_a pos_1_b [TAB]"),
        snapbox::str![[r#"
pos_1_a
pos_1_b
pos_1_c
--format
--help	Print help
"#]]
    );
    assert_data_eq!(
        complete!(cmd, "pos_1_a pos_1_b --[TAB]"),
        snapbox::str![[r#"
--format
--help	Print help
"#]]
    );
    assert_data_eq!(
        complete!(cmd, "pos_1_a pos_1_b --format [TAB]"),
        snapbox::str![[r#"
json
yaml
toml
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "pos_1_a pos_1_b --format json [TAB]"),
        snapbox::str![[r#"
pos_1_a
pos_1_b
pos_1_c
--format
--help	Print help
"#]]
    );
}

#[test]
fn suggest_delimiter_values() {
    let mut cmd = Command::new("delimiter")
        .arg(
            clap::Arg::new("delimiter")
                .long("delimiter")
                .short('D')
                .value_parser([
                    PossibleValue::new("comma"),
                    PossibleValue::new("space"),
                    PossibleValue::new("tab"),
                ])
                .value_delimiter(','),
        )
        .arg(
            clap::Arg::new("pos")
                .value_parser(["a_pos", "b_pos", "c_pos"])
                .value_delimiter(','),
        );

    assert_data_eq!(
        complete!(cmd, "--delimiter [TAB]"),
        snapbox::str![[r#"
comma
space
tab
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--delimiter=[TAB]"),
        snapbox::str![[r#"
--delimiter=comma
--delimiter=space
--delimiter=tab
"#]]
    );

    assert_data_eq!(complete!(cmd, "--delimiter c[TAB]"), snapbox::str!["comma"]);

    assert_data_eq!(
        complete!(cmd, "--delimiter=c[TAB]"),
        snapbox::str!["--delimiter=comma"]
    );

    assert_data_eq!(
        complete!(cmd, "--delimiter comma,[TAB]"),
        snapbox::str![[r#"
comma,comma
comma,space
comma,tab
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--delimiter=comma,[TAB]"),
        snapbox::str![[r#"
--delimiter=comma,a_pos
--delimiter=comma,b_pos
--delimiter=comma,c_pos
--delimiter=comma,comma
--delimiter=comma,space
--delimiter=comma,tab
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--delimiter comma,s[TAB]"),
        snapbox::str!["comma,space"]
    );

    assert_data_eq!(
        complete!(cmd, "--delimiter=comma,s[TAB]"),
        snapbox::str!["--delimiter=comma,space"]
    );

    assert_data_eq!(
        complete!(cmd, "-D [TAB]"),
        snapbox::str![[r#"
comma
space
tab
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-D=[TAB]"),
        snapbox::str![[r#"
-D=comma
-D=space
-D=tab
"#]]
    );

    assert_data_eq!(complete!(cmd, "-D c[TAB]"), snapbox::str!["comma"]);

    assert_data_eq!(complete!(cmd, "-D=c[TAB]"), snapbox::str!["-D=comma"]);

    assert_data_eq!(
        complete!(cmd, "-D comma,[TAB]"),
        snapbox::str![[r#"
comma,comma
comma,space
comma,tab
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-D=comma,[TAB]"),
        snapbox::str![[r#"
-D=comma,a_pos
-D=comma,b_pos
-D=comma,c_pos
-D=comma,comma
-D=comma,space
-D=comma,tab
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-D comma,s[TAB]"),
        snapbox::str!["comma,space"]
    );

    assert_data_eq!(
        complete!(cmd, "-D=comma,s[TAB]"),
        snapbox::str!["-D=comma,space"]
    );

    assert_data_eq!(
        complete!(cmd, "-- [TAB]"),
        snapbox::str![[r#"
a_pos
b_pos
c_pos
"#]]
    );

    assert_data_eq!(
        complete!(cmd, " -- a_pos,[TAB]"),
        snapbox::str![[r#"
a_pos,a_pos
a_pos,b_pos
a_pos,c_pos
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-- a_pos,b[TAB]"),
        snapbox::str!["a_pos,b_pos"]
    );
}

#[test]
fn suggest_allow_hyphen() {
    let mut cmd = Command::new("exhaustive")
        .arg(
            clap::Arg::new("format")
                .long("format")
                .short('F')
                .allow_hyphen_values(true)
                .value_parser(["--json", "--toml", "--yaml"]),
        )
        .arg(clap::Arg::new("json").long("json"));

    assert_data_eq!(complete!(cmd, "--format --j[TAB]"), snapbox::str!["--json"]);
    assert_data_eq!(complete!(cmd, "-F --j[TAB]"), snapbox::str!["--json"]);
    assert_data_eq!(complete!(cmd, "--format --t[TAB]"), snapbox::str!["--toml"]);
    assert_data_eq!(complete!(cmd, "-F --t[TAB]"), snapbox::str!["--toml"]);

    assert_data_eq!(
        complete!(cmd, "--format --[TAB]"),
        snapbox::str![[r#"
--json
--toml
--yaml
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "-F --[TAB]"),
        snapbox::str![[r#"
--json
--toml
--yaml
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--format --json --j[TAB]"),
        snapbox::str!["--json"]
    );

    assert_data_eq!(
        complete!(cmd, "-F --json --j[TAB]"),
        snapbox::str!["--json"]
    );
}

#[test]
fn suggest_positional_long_allow_hyphen() {
    let mut cmd = Command::new("exhaustive")
        .arg(
            clap::Arg::new("format")
                .long("format")
                .short('F')
                .allow_hyphen_values(true)
                .value_parser(["--json", "--toml", "--yaml"]),
        )
        .arg(
            clap::Arg::new("positional_a")
                .value_parser(["--pos_a"])
                .allow_hyphen_values(true),
        )
        .arg(clap::Arg::new("positional_b").value_parser(["pos_b"]));

    assert_data_eq!(
        complete!(cmd, "--format --json --pos[TAB]"),
        snapbox::str!["--pos_a"]
    );
    assert_data_eq!(
        complete!(cmd, "-F --json --pos[TAB]"),
        snapbox::str!["--pos_a"]
    );

    assert_data_eq!(
        complete!(cmd, "--format --json --pos_a [TAB]"),
        snapbox::str![[r#"
pos_b
--format
--help	Print help
"#]]
    );
    assert_data_eq!(
        complete!(cmd, "-F --json --pos_a [TAB]"),
        snapbox::str![[r#"
pos_b
--format
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--format --json --pos_a p[TAB]"),
        snapbox::str!["pos_b"]
    );
    assert_data_eq!(
        complete!(cmd, "-F --json --pos_a p[TAB]"),
        snapbox::str!["pos_b"]
    );
}

#[test]
fn suggest_positional_short_allow_hyphen() {
    let mut cmd = Command::new("exhaustive")
        .arg(
            clap::Arg::new("format")
                .long("format")
                .short('F')
                .allow_hyphen_values(true)
                .value_parser(["--json", "--toml", "--yaml"]),
        )
        .arg(
            clap::Arg::new("positional_a")
                .value_parser(["-a"])
                .allow_hyphen_values(true),
        )
        .arg(clap::Arg::new("positional_b").value_parser(["pos_b"]));

    assert_data_eq!(
        complete!(cmd, "--format --json -a [TAB]"),
        snapbox::str![[r#"
pos_b
--format
--help	Print help
"#]]
    );
    assert_data_eq!(
        complete!(cmd, "-F --json -a [TAB]"),
        snapbox::str![[r#"
pos_b
--format
--help	Print help
"#]]
    );

    assert_data_eq!(
        complete!(cmd, "--format --json -a p[TAB]"),
        snapbox::str!["pos_b"]
    );
    assert_data_eq!(
        complete!(cmd, "-F --json -a p[TAB]"),
        snapbox::str!["pos_b"]
    );
}

#[test]
fn suggest_external_subcommand() {
    let mut cmd = Command::new("dynamic")
        .allow_external_subcommands(true)
        .add(SubcommandCandidates::new(|| {
            vec![CompletionCandidate::new("external")]
        }))
        .arg(clap::Arg::new("positional").value_parser(["pos1", "pos2", "pos3"]));

    assert_data_eq!(
        complete!(cmd, " [TAB]"),
        snapbox::str![
            "external
pos1
pos2
pos3
--help\tPrint help
"
        ]
    );

    assert_data_eq!(complete!(cmd, "e[TAB]"), snapbox::str!["external"]);
}

fn tool_cmd() -> Command {
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .args_override_self(true)
        .arg(
            clap::Arg::new("json")
                .long("json")
                .visible_alias("j")
                .alias("legacy-json")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with("toml"),
        )
        .arg(
            clap::Arg::new("toml")
                .long("toml")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("verbose")
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with("quiet"),
        )
        .arg(
            clap::Arg::new("quiet")
                .long("quiet")
                .action(clap::ArgAction::SetTrue),
        )
}

fn complete_tool(args: &[&str]) -> Vec<String> {
    let mut cmd = tool_cmd();
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(&mut cmd, args, arg_index, None)
        .unwrap()
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn conflicting_long_options_are_filtered() {
    // Baseline: no conflicts in play, grouping and order are untouched.
    assert_eq!(
        complete_tool(&["tool", ""]),
        ["--json", "--j", "--toml", "--verbose", "--quiet"]
    );

    // `--json` conflicts with `--toml`; unrelated candidates remain.
    let completions = complete_tool(&["tool", "--json", ""]);
    assert!(completions.contains(&"--verbose".to_owned()));
    assert!(completions.contains(&"--quiet".to_owned()));
    assert!(!completions.contains(&"--toml".to_owned()));
    assert_eq!(completions, ["--json", "--j", "--verbose", "--quiet"]);
}

#[test]
fn conflict_filter_respects_prefix() {
    // `--t` only prefixes `--toml`, which conflicts with `--json`;
    // the result is a successful empty completion, not an unrelated candidate.
    let completions = complete_tool(&["tool", "--json", "--t"]);
    assert!(!completions.contains(&"--toml".to_owned()));
    assert!(!completions.contains(&"--quiet".to_owned()));
    assert!(completions.is_empty());
}

#[test]
fn conflict_filter_with_repeated_flag() {
    // Repeating the same flag does not change the filtered result.
    let completions = complete_tool(&["tool", "--json", "--json", ""]);
    assert!(completions.contains(&"--verbose".to_owned()));
    assert!(completions.contains(&"--quiet".to_owned()));
    assert!(!completions.contains(&"--toml".to_owned()));
    assert_eq!(completions, ["--json", "--j", "--verbose", "--quiet"]);
}

#[test]
fn conflict_is_bidirectional() {
    // The conflict is only declared on `json`, but `--toml` on the
    // command line must also exclude `--json`.
    let completions = complete_tool(&["tool", "--toml", ""]);
    assert!(completions.contains(&"--verbose".to_owned()));
    assert!(completions.contains(&"--quiet".to_owned()));
    assert!(!completions.contains(&"--json".to_owned()));
    assert_eq!(completions, ["--toml", "--verbose", "--quiet"]);
}

#[test]
fn conflicting_prior_args_do_not_fail_completion() {
    // The command line already contains a conflicting pair; completion
    // must still succeed and offer the remaining non-conflicting candidates.
    let completions = complete_tool(&["tool", "--verbose", "--quiet", ""]);
    assert!(completions.contains(&"--json".to_owned()));
    assert!(completions.contains(&"--toml".to_owned()));
    assert_eq!(completions, ["--json", "--j", "--toml"]);
}

#[test]
fn visible_alias_shares_conflicts() {
    // `--j` is a visible alias of `json`, so `toml` is excluded.
    let completions = complete_tool(&["tool", "--j", ""]);
    assert!(completions.contains(&"--verbose".to_owned()));
    assert!(completions.contains(&"--quiet".to_owned()));
    assert!(!completions.contains(&"--toml".to_owned()));
}

#[test]
fn hidden_alias_shares_conflicts() {
    // `--legacy-json` is a hidden alias of `json`; combined with `--toml`
    // on the command line, `json` must be excluded from the candidates.
    let completions = complete_tool(&["tool", "--legacy-json", "--toml", ""]);
    assert!(completions.contains(&"--verbose".to_owned()));
    assert!(completions.contains(&"--quiet".to_owned()));
    assert!(!completions.contains(&"--json".to_owned()));
    assert_eq!(completions, ["--verbose", "--quiet"]);
}

/// Build the `tool` command used to verify dynamic completion of a required,
/// non-multiple argument group.
///
/// Root:
/// - a required, exclusive `format` group of the value-less `--json`/`--yaml`
///   flags, with visible aliases `--js`/`--yml`
/// - `--output`, restricted to `file`/`stream`
/// - the `run` subcommand
///
/// `run` has its own required, exclusive `mode` group (`--text`/`--binary`)
/// and a positional with the values `job`/`log`.
fn exclusive_group_cmd() -> Command {
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(
            clap::Arg::new("json")
                .long("json")
                .visible_alias("js")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("yaml")
                .long("yaml")
                .visible_alias("yml")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("output")
                .long("output")
                .value_parser(["file", "stream"]),
        )
        .group(
            ArgGroup::new("format")
                .args(["json", "yaml"])
                .required(true)
                .multiple(false),
        )
        .subcommand(
            Command::new("run")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .disable_help_subcommand(true)
                .arg(
                    clap::Arg::new("text")
                        .long("text")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::Arg::new("binary")
                        .long("binary")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(clap::Arg::new("kind").value_parser(["job", "log"]))
                .group(
                    ArgGroup::new("mode")
                        .args(["text", "binary"])
                        .required(true)
                        .multiple(false),
                ),
        )
}

fn complete_exclusive_group(cmd: &mut Command, args: &[&str]) -> Vec<CompletionCandidate> {
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(cmd, args, arg_index, None).unwrap()
}

fn exclusive_group_values(cmd: &mut Command, args: &[&str]) -> Vec<String> {
    complete_exclusive_group(cmd, args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn exclusive_group_empty_lists_members_aliases_and_unrelated() {
    // Before a choice is made the empty word offers both members' primary
    // names and visible aliases (four entries), plus the unrelated `--output`
    // option and the `run` subcommand.
    let mut cmd = exclusive_group_cmd();
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", ""]),
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );
}

#[test]
fn exclusive_group_choice_hides_the_whole_group() {
    // After any member (or its visible alias) is chosen, every primary name
    // and alias of the same group disappears, while unrelated candidates and
    // the subcommand stay available.
    for chosen in ["--json", "--js", "--yaml", "--yml"] {
        let mut cmd = exclusive_group_cmd();
        assert_eq!(
            exclusive_group_values(&mut cmd, &["tool", chosen, ""]),
            ["run", "--output"],
            "choice {chosen}"
        );
    }
}

#[test]
fn exclusive_group_repeated_choice_still_succeeds() {
    // Repeating the selected member (under either spelling) must succeed,
    // must not bring back any group candidate, and must not error.
    for prior in [
        &["tool", "--json", "--json", ""][..],
        &["tool", "--json", "--js", ""][..],
        &["tool", "--js", "--json", ""][..],
        &["tool", "--yaml", "--yml", ""][..],
    ] {
        let mut cmd = exclusive_group_cmd();
        assert_eq!(exclusive_group_values(&mut cmd, prior), ["run", "--output"]);
    }
}

#[test]
fn exclusive_group_prefix_only_offers_members() {
    // A member prefix before any choice narrows the group down to matching
    // spellings; unrelated options are not offered for that prefix.
    let mut cmd = exclusive_group_cmd();
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "--j"]),
        ["--json", "--js"]
    );

    // Once a choice is made a prefix that only matched the chosen group is
    // an empty success instead of restoring a group candidate.
    let mut cmd = exclusive_group_cmd();
    assert!(exclusive_group_values(&mut cmd, &["tool", "--json", "--y"]).is_empty());
    let mut cmd = exclusive_group_cmd();
    assert!(exclusive_group_values(&mut cmd, &["tool", "--yaml", "--j"]).is_empty());
}

#[test]
fn exclusive_group_output_values_are_completed() {
    // The value-taking `--output` option is independent of the format group.
    let mut cmd = exclusive_group_cmd();
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "--output", ""]),
        ["file", "stream"]
    );

    let mut cmd = exclusive_group_cmd();
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "--output", "f"]),
        ["file"]
    );

    // `--output=` followed by a separate word is the same pending-value
    // state as `--output`, and prefix filtering still applies.
    let mut cmd = exclusive_group_cmd();
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "--output=", "s"]),
        ["stream"]
    );
}

#[test]
fn exclusive_group_invalid_output_value_does_not_pollute() {
    // An illegal attached value must not error and must not leave `--output`
    // in a stuck state: the following empty word offers the normal root set.
    let mut cmd = exclusive_group_cmd();
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "--output=pipe", ""]),
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );

    // A later completion on the same command object is unaffected.
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "--output", ""]),
        ["file", "stream"]
    );
}

#[test]
fn exclusive_group_does_not_leak_into_subcommand() {
    // Inside `run` only the run-level group members and positional values are
    // offered; the root group and root options never leak down.
    let mut cmd = exclusive_group_cmd();
    let values = exclusive_group_values(&mut cmd, &["tool", "run", ""]);
    assert_eq!(values, ["job", "log", "--text", "--binary"]);
    for leaked in ["--json", "--js", "--yaml", "--yml", "--output"] {
        assert!(!values.contains(&leaked.to_owned()), "`{leaked}` leaked into run");
    }
}

#[test]
fn exclusive_group_subcommand_choice_hides_its_own_group() {
    // The run-level group behaves like the root group: choosing a member
    // leaves only the positional values.
    for chosen in ["--text", "--binary"] {
        let mut cmd = exclusive_group_cmd();
        assert_eq!(
            exclusive_group_values(&mut cmd, &["tool", "run", chosen, ""]),
            ["job", "log"],
            "choice {chosen}"
        );
    }

    // Repeating the choice is fine too.
    let mut cmd = exclusive_group_cmd();
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "run", "--text", "--text", ""]),
        ["job", "log"]
    );
}

#[test]
fn exclusive_group_subcommand_positional_prefixes() {
    let mut cmd = exclusive_group_cmd();
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "run", "j"]),
        ["job"]
    );

    let mut cmd = exclusive_group_cmd();
    assert!(exclusive_group_values(&mut cmd, &["tool", "run", "x"]).is_empty());

    // After a member choice, the positional prefixes still complete while the
    // group stays hidden.
    let mut cmd = exclusive_group_cmd();
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "run", "--binary", "l"]),
        ["log"]
    );
}

#[test]
fn exclusive_group_aliases_and_invalid_prefixes_are_empty_successes() {
    // Unknown options and no-match prefixes complete successfully with an
    // empty set rather than erroring.
    for args in [
        &["tool", "--bogus"][..],
        &["tool", "--json", "--bogus"][..],
        &["tool", "run", "--bogus"][..],
        &["tool", "zzzzz"][..],
        &["tool", "run", "zzzzz"][..],
    ] {
        let mut cmd = exclusive_group_cmd();
        assert!(
            exclusive_group_values(&mut cmd, args).is_empty(),
            "args {args:?} should be empty"
        );
    }
}

#[test]
fn exclusive_group_state_is_not_polluted_across_calls() {
    // Every call shares the same command object; choices and invalid input in
    // one call must not affect the candidates of later calls.
    let mut cmd = exclusive_group_cmd();

    // A choice filters the group for that call only.
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "--json", ""]),
        ["run", "--output"]
    );
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", ""]),
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );

    // Failed/illegal input leaves the command usable.
    assert!(exclusive_group_values(&mut cmd, &["tool", "--json", "--yaml", "--zzz"]).is_empty());
    assert!(exclusive_group_values(&mut cmd, &["tool", "--output=pipe", "f"]).is_empty());
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "--yaml", ""]),
        ["run", "--output"]
    );
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", "run", "--text", ""]),
        ["job", "log"]
    );
    // A root completion after a run completion is independent.
    assert_eq!(
        exclusive_group_values(&mut cmd, &["tool", ""]),
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );
}

#[test]
fn exclusive_group_preserves_order_and_metadata_of_unfiltered() {
    // Candidates not removed by the group filter keep their relative order and
    // full metadata (value, help, id, tag, display order, hidden state).
    type CandidateMetadata = (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<usize>,
        bool,
    );
    fn metadata(candidates: Vec<CompletionCandidate>) -> Vec<CandidateMetadata> {
        candidates
            .into_iter()
            .map(|candidate| {
                (
                    candidate.get_value().to_string_lossy().into_owned(),
                    candidate.get_help().map(|help| help.to_string()),
                    candidate.get_id().cloned(),
                    candidate.get_tag().map(|tag| tag.to_string()),
                    candidate.get_display_order(),
                    candidate.is_hide_set(),
                )
            })
            .collect()
    }

    let mut baseline_cmd = exclusive_group_cmd();
    let baseline = metadata(complete_exclusive_group(&mut baseline_cmd, &["tool", ""]));
    let expected: Vec<_> = baseline
        .into_iter()
        .filter(|candidate| candidate.0 == "run" || candidate.0 == "--output")
        .collect();

    for chosen in ["--json", "--js", "--yaml", "--yml"] {
        let mut cmd = exclusive_group_cmd();
        assert_eq!(
            metadata(complete_exclusive_group(&mut cmd, &["tool", chosen, ""])),
            expected,
            "choice {chosen}"
        );
    }
}

fn terminator_tool_cmd() -> Command {
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(
            clap::Arg::new("config")
                .long("config")
                .value_parser(["dev", "prod"]),
        )
        .arg(clap::Arg::new("root-pos").value_parser(["alpha", "beta"]))
        .subcommand(
            Command::new("run")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .arg(
                    clap::Arg::new("format")
                        .long("format")
                        .value_parser(["text", "json"]),
                )
                .arg(clap::Arg::new("run-pos").value_parser(["one", "two"])),
        )
}

fn complete_terminator_tool(args: &[&str]) -> Vec<CompletionCandidate> {
    let mut cmd = terminator_tool_cmd();
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(&mut cmd, args, arg_index, None).unwrap()
}

fn terminator_tool_values(args: &[&str]) -> Vec<String> {
    complete_terminator_tool(args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn pending_option_value_completes_only_its_values() {
    assert_eq!(
        terminator_tool_values(&["tool", "--config", ""]),
        ["dev", "prod"]
    );
    assert_eq!(
        terminator_tool_values(&["tool", "run", "--format", ""]),
        ["text", "json"]
    );
}

#[test]
fn pending_option_value_with_flag_like_prefix_is_empty() {
    // A flag-like word while an option value is pending must succeed with no
    // candidates, not leak options, subcommands, or positional values.
    assert!(terminator_tool_values(&["tool", "--config", "--f"]).is_empty());
    assert!(terminator_tool_values(&["tool", "run", "--format", "--f"]).is_empty());
}

#[test]
fn terminator_after_pending_option_value_completes_positionals() {
    // `--` cancels the pending option value; only positionals are offered,
    // not the pending option, other options, or subcommands.
    assert_eq!(
        terminator_tool_values(&["tool", "--config", "--", ""]),
        ["alpha", "beta"]
    );
    assert_eq!(
        terminator_tool_values(&["tool", "run", "--format", "--", ""]),
        ["one", "two"]
    );
}

#[test]
fn flag_like_word_after_terminator_is_empty() {
    assert!(terminator_tool_values(&["tool", "--config", "--", "--f"]).is_empty());
    assert!(terminator_tool_values(&["tool", "run", "--format", "--", "--f"]).is_empty());
}

#[test]
fn pre_terminator_behavior_is_unchanged() {
    assert_eq!(terminator_tool_values(&["tool", "--c"]), ["--config"]);
    assert_eq!(terminator_tool_values(&["tool", "--config", "d"]), ["dev"]);
    assert_eq!(terminator_tool_values(&["tool", "run", "--f"]), ["--format"]);
    assert_eq!(
        terminator_tool_values(&["tool", "run", "--format", "j"]),
        ["json"]
    );
}

#[test]
fn root_terminator_completes_only_root_positionals() {
    // No options, no subcommands, only the root positional values.
    assert_eq!(terminator_tool_values(&["tool", "--", ""]), ["alpha", "beta"]);
}

#[test]
fn nested_terminator_completes_only_nested_positionals() {
    // The terminator inside `run` stays at the `run` level: its positionals
    // are offered, not the root positionals or root options.
    assert_eq!(terminator_tool_values(&["tool", "run", "--", ""]), ["one", "two"]);
}

#[test]
fn subcommand_name_after_terminator_does_not_descend() {
    // `run` after `--` is a positional value, not a subcommand: parsing must
    // not descend into `run`, so no `run`-level candidates are offered.
    assert!(terminator_tool_values(&["tool", "--", "run", ""]).is_empty());
    // Completing the subcommand-named word itself offers nothing either.
    assert!(terminator_tool_values(&["tool", "--", "run"]).is_empty());
    assert!(terminator_tool_values(&["tool", "--", "r"]).is_empty());
    assert!(terminator_tool_values(&["tool", "run", "--", "run"]).is_empty());
}

#[test]
fn option_like_words_after_terminator_are_empty() {
    // Option-style words after `--` are positional values; none match the
    // possible values, so completion succeeds with no candidates instead of
    // leaking options from the current or parent command.
    for word in ["--f", "--config", "--format", "-x"] {
        assert!(terminator_tool_values(&["tool", "--", word]).is_empty());
        assert!(terminator_tool_values(&["tool", "run", "--", word]).is_empty());
    }
}

#[test]
fn prefix_after_terminator_filters_positionals() {
    assert_eq!(terminator_tool_values(&["tool", "--", "a"]), ["alpha"]);
    assert_eq!(terminator_tool_values(&["tool", "--", "b"]), ["beta"]);
    assert_eq!(terminator_tool_values(&["tool", "--config", "--", "a"]), ["alpha"]);
    // The pending `--config` value is cancelled by `--`; its values (`prod`)
    // must not leak into the root positional completion.
    assert!(terminator_tool_values(&["tool", "--config", "--", "p"]).is_empty());
    assert_eq!(terminator_tool_values(&["tool", "run", "--", "t"]), ["two"]);
    assert_eq!(
        terminator_tool_values(&["tool", "run", "--format", "--", "o"]),
        ["one"]
    );
}

#[test]
fn invalid_words_before_terminator_still_complete() {
    // Unknown options before `--` must not break completion after the
    // terminator; the current level's positionals are still offered.
    assert_eq!(
        terminator_tool_values(&["tool", "--bogus", "--", ""]),
        ["alpha", "beta"]
    );
    assert_eq!(
        terminator_tool_values(&["tool", "run", "--bogus", "--", ""]),
        ["one", "two"]
    );
}

#[test]
fn conflicting_args_before_terminator_still_complete() {
    // A conflicting pair already on the command line must not make completion
    // after the terminator fail or drop the current level's positionals.
    let mut cmd = terminator_tool_cmd()
        .arg(
            clap::Arg::new("alpha-flag")
                .long("aa")
                .conflicts_with("beta-flag"),
        )
        .arg(clap::Arg::new("beta-flag").long("bb"));
    let mut complete = |args: &[&str]| {
        let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
        let arg_index = args.len() - 1;
        clap_complete::engine::complete(&mut cmd, args, arg_index, None)
            .unwrap()
            .into_iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    };

    assert_eq!(complete(&["tool", "--aa", "--bb", "--", ""]), ["alpha", "beta"]);
    assert_eq!(
        complete(&["tool", "--aa", "--bb", "--config", "--", ""]),
        ["alpha", "beta"]
    );
}

#[test]
fn out_of_range_positional_after_terminator_is_empty() {
    // Positional slots are exhausted; completion must succeed with no
    // candidates rather than erroring or falling back to the parent command.
    assert!(terminator_tool_values(&["tool", "--", "alpha", ""]).is_empty());
    assert!(terminator_tool_values(&["tool", "run", "--", "one", ""]).is_empty());
}

#[test]
fn terminator_preserves_candidate_metadata() {
    // Cancelling the pending option value may only shrink the candidate set;
    // the retained candidates must be byte-for-byte identical in value, help,
    // id, tag, display order, and hidden state.
    type CandidateMetadata = (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<usize>,
        bool,
    );
    fn metadata(candidates: Vec<CompletionCandidate>) -> Vec<CandidateMetadata> {
        candidates
            .into_iter()
            .map(|candidate| {
                (
                    candidate.get_value().to_string_lossy().into_owned(),
                    candidate.get_help().map(|help| help.to_string()),
                    candidate.get_id().cloned(),
                    candidate.get_tag().map(|tag| tag.to_string()),
                    candidate.get_display_order(),
                    candidate.is_hide_set(),
                )
            })
            .collect()
    }

    let plain = metadata(complete_terminator_tool(&["tool", ""]));
    let escaped = metadata(complete_terminator_tool(&["tool", "--config", "--", ""]));
    let expected: Vec<_> = plain
        .iter()
        .filter(|candidate| candidate.0 == "alpha" || candidate.0 == "beta")
        .cloned()
        .collect();
    assert_eq!(escaped, expected);

    let escaped = metadata(complete_terminator_tool(&["tool", "--", ""]));
    assert_eq!(escaped, expected);

    let plain = metadata(complete_terminator_tool(&["tool", "run", ""]));
    let escaped = metadata(complete_terminator_tool(&[
        "tool", "run", "--format", "--", "",
    ]));
    let expected: Vec<_> = plain
        .iter()
        .filter(|candidate| candidate.0 == "one" || candidate.0 == "two")
        .cloned()
        .collect();
    assert_eq!(escaped, expected);

    let escaped = metadata(complete_terminator_tool(&["tool", "run", "--", ""]));
    assert_eq!(escaped, expected);
}

fn alias_tool_cmd() -> Command {
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(clap::Arg::new("root-pos").value_parser(["config", "status"]))
        .subcommand(
            Command::new("run")
                .visible_alias("r")
                .alias("legacy-run")
                .about("Run a job")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .disable_help_subcommand(true)
                .arg(clap::Arg::new("run-pos").value_parser(["job", "log"]))
                .arg(
                    clap::Arg::new("env")
                        .long("env")
                        .value_parser(["dev", "prod"])
                        .help("Target environment"),
                )
                .subcommand(
                    Command::new("deploy")
                        .visible_alias("d")
                        .about("Deploy a plan")
                        .disable_help_flag(true)
                        .disable_version_flag(true)
                        .disable_help_subcommand(true)
                        .arg(clap::Arg::new("deploy-pos").value_parser(["plan", "force"]))
                        .arg(
                            clap::Arg::new("target")
                                .long("target")
                                .value_parser(["local", "remote"])
                                .help("Deploy target"),
                        ),
                ),
        )
        .subcommand(
            Command::new("inspect")
                .visible_alias("i")
                .about("Inspect state")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .disable_help_subcommand(true),
        )
}

fn complete_alias_tool(args: &[&str]) -> Vec<CompletionCandidate> {
    let mut cmd = alias_tool_cmd();
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(&mut cmd, args, arg_index, None).unwrap()
}

fn alias_tool_values(args: &[&str]) -> Vec<String> {
    complete_alias_tool(args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

/// Full candidate metadata, so that equivalence between a canonical path and
/// its alias paths covers value, help, id, tag, display order, and hidden
/// state, not just the displayed value.
fn alias_tool_metadata(args: &[&str]) -> Vec<CandidateMetadata> {
    candidate_metadata(complete_alias_tool(args))
}

#[test]
fn root_empty_lists_names_and_visible_aliases() {
    // Both primary names and their visible aliases are listed; the hidden
    // alias `legacy-run` must not appear.
    let values = alias_tool_values(&["tool", ""]);
    assert_eq!(
        values,
        ["run", "r", "inspect", "i", "config", "status"]
    );
    assert!(!values.contains(&"legacy-run".to_owned()));
}

#[test]
fn root_name_prefix_returns_only_matches() {
    assert_eq!(alias_tool_values(&["tool", "r"]), ["run", "r"]);
    assert_eq!(alias_tool_values(&["tool", "ru"]), ["run"]);
    assert_eq!(alias_tool_values(&["tool", "i"]), ["inspect", "i"]);
    assert_eq!(alias_tool_values(&["tool", "in"]), ["inspect"]);
    // A prefix matching only the hidden alias still offers it, as no visible
    // candidate matches.
    assert_eq!(alias_tool_values(&["tool", "l"]), ["legacy-run"]);
}

#[test]
fn visible_alias_descent_matches_canonical_path() {
    // Entering `run` through its visible alias must offer the same candidates,
    // with the same metadata, as the canonical path.
    let canonical = alias_tool_metadata(&["tool", "run", ""]);
    assert_eq!(
        canonical,
        alias_tool_metadata(&["tool", "r", ""])
    );
    let values: Vec<_> = canonical.iter().map(|candidate| candidate.0.clone()).collect();
    assert_eq!(values, ["deploy", "d", "job", "log", "--env"]);
}

#[test]
fn hidden_alias_descent_matches_canonical_path() {
    // A fully typed hidden alias is a valid entry point into the subcommand.
    let canonical = alias_tool_metadata(&["tool", "run", ""]);
    assert_eq!(
        canonical,
        alias_tool_metadata(&["tool", "legacy-run", ""])
    );
}

#[test]
fn descended_scope_excludes_parent_and_siblings() {
    // After descending, the parent level's positionals, sibling subcommands,
    // and the hidden alias must not leak into the candidates.
    let values = alias_tool_values(&["tool", "r", ""]);
    for leaked in ["config", "status", "inspect", "i", "run", "legacy-run"] {
        assert!(
            !values.contains(&leaked.to_owned()),
            "`{leaked}` must not be offered inside `run`"
        );
    }
}

#[test]
fn nested_alias_descent_matches_canonical_path() {
    // `r d`, `run d`, `r deploy`, and `run deploy` are equivalent paths into
    // the `deploy` level.
    let canonical = alias_tool_metadata(&["tool", "run", "deploy", ""]);
    for path in [
        &["tool", "r", "d", ""][..],
        &["tool", "run", "d", ""][..],
        &["tool", "r", "deploy", ""][..],
        &["tool", "legacy-run", "d", ""][..],
    ] {
        assert_eq!(canonical, alias_tool_metadata(path), "path {path:?}");
    }
    let values: Vec<_> = canonical.iter().map(|candidate| candidate.0.clone()).collect();
    assert_eq!(values, ["plan", "force", "--target"]);
}

#[test]
fn option_values_under_alias_path() {
    // Space-separated and `=`-attached option values behave the same under an
    // alias path as on the canonical path, keeping candidates and metadata.
    for args in [
        &["tool", "r", "--env", ""][..],
        &["tool", "legacy-run", "--env", ""][..],
    ] {
        assert_eq!(alias_tool_values(args), ["dev", "prod"]);
        assert_eq!(
            alias_tool_metadata(args),
            alias_tool_metadata(&["tool", "run", "--env", ""])
        );
    }
    assert_eq!(alias_tool_values(&["tool", "r", "--env", "p"]), ["prod"]);
    assert_eq!(
        alias_tool_values(&["tool", "r", "--env="]),
        ["--env=dev", "--env=prod"]
    );
    assert_eq!(
        alias_tool_metadata(&["tool", "r", "--env="]),
        alias_tool_metadata(&["tool", "run", "--env="])
    );
    assert_eq!(
        alias_tool_values(&["tool", "r", "--env=p"]),
        ["--env=prod"]
    );

    // One level down, through two aliases.
    assert_eq!(
        alias_tool_values(&["tool", "r", "d", "--target", ""]),
        ["local", "remote"]
    );
    assert_eq!(
        alias_tool_metadata(&["tool", "r", "d", "--target", ""]),
        alias_tool_metadata(&["tool", "run", "deploy", "--target", ""])
    );
    assert_eq!(
        alias_tool_values(&["tool", "r", "d", "--target="]),
        ["--target=local", "--target=remote"]
    );
    assert_eq!(
        alias_tool_values(&["tool", "r", "d", "--target", "r"]),
        ["remote"]
    );
}

#[test]
fn terminator_at_alias_level_completes_only_positionals() {
    // `--` at the current (alias-entered) level leaves only that level's
    // positional values.
    assert_eq!(alias_tool_values(&["tool", "--", ""]), ["config", "status"]);
    assert_eq!(alias_tool_values(&["tool", "r", "--", ""]), ["job", "log"]);
    assert_eq!(
        alias_tool_metadata(&["tool", "r", "--", ""]),
        alias_tool_metadata(&["tool", "run", "--", ""])
    );
    assert_eq!(
        alias_tool_values(&["tool", "r", "d", "--", ""]),
        ["plan", "force"]
    );
}

#[test]
fn words_after_terminator_do_not_switch_levels() {
    // After `--`, aliases, option-style words, and unknown words are
    // positional values; none match, so completion succeeds with empty
    // candidates instead of re-switching or falling back to another level.
    assert!(alias_tool_values(&["tool", "r", "--", "d"]).is_empty());
    assert!(alias_tool_values(&["tool", "r", "--", "deploy"]).is_empty());
    assert!(alias_tool_values(&["tool", "r", "--", "--env"]).is_empty());
    assert!(alias_tool_values(&["tool", "r", "--", "--e"]).is_empty());
    assert!(alias_tool_values(&["tool", "r", "--", "bogus"]).is_empty());
    // A subcommand name after the terminator must not descend.
    assert!(alias_tool_values(&["tool", "--", "run", ""]).is_empty());
    assert!(alias_tool_values(&["tool", "r", "--", "d", ""]).is_empty());
}

#[test]
fn invalid_alias_prefix_is_empty_and_does_not_pollute() {
    // An invalid name prefix successfully returns no candidates ...
    let mut cmd = alias_tool_cmd();
    let complete = |cmd: &mut Command, args: &[&str]| {
        let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
        let arg_index = args.len() - 1;
        clap_complete::engine::complete(cmd, args, arg_index, None)
            .unwrap()
            .into_iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    };
    assert!(complete(&mut cmd, &["tool", "rx"]).is_empty());
    assert!(complete(&mut cmd, &["tool", "run", "rx"]).is_empty());

    // ... and later completions on the same command are unaffected.
    assert_eq!(
        complete(&mut cmd, &["tool", ""]),
        ["run", "r", "inspect", "i", "config", "status"]
    );
    assert_eq!(
        complete(&mut cmd, &["tool", "r", ""]),
        ["deploy", "d", "job", "log", "--env"]
    );
}

#[test]
fn sort_and_filter() {
    let mut cmd = Command::new("exhaustive")
        .args([
            clap::Arg::new("required-flag")
                .long("required-flag")
                .visible_alias("required-flag2")
                .short('r')
                .required(true),
            clap::Arg::new("optional-flag")
                .long("optional-flag")
                .visible_alias("2optional-flag")
                .short('o'),
            clap::Arg::new("long-flag").long("long-flag"),
            clap::Arg::new("short-flag").short('s'),
            clap::Arg::new("positional").value_parser(["pos-a", "pos-b", "pos-c"]),
        ])
        .subcommands([Command::new("sub")]);

    assert_data_eq!(
        complete!(cmd, " [TAB]"),
        snapbox::str![[r#"
sub
help	Print this message or the help of the given subcommand(s)
pos-a
pos-b
pos-c
--required-flag
--required-flag2
--optional-flag
--2optional-flag
--long-flag
-s
--help	Print help
"#]]
    );
    assert_data_eq!(
        complete!(cmd, "-[TAB]"),
        snapbox::str![[r#"
-r	--required-flag
-o	--optional-flag
--long-flag
-s
-h	Print help
"#]]
    );
    assert_data_eq!(
        complete!(cmd, "--[TAB]"),
        snapbox::str![[r#"
--required-flag
--required-flag2
--optional-flag
--2optional-flag
--long-flag
--help	Print help
"#]]
    );
}

fn multi_value_tag_cmd() -> Command {
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(
            clap::Arg::new("tag")
                .long("tag")
                .num_args(1..=3)
                .value_parser(["red", "blue", "green"]),
        )
        .arg(
            clap::Arg::new("limit")
                .long("limit")
                .num_args(1)
                .value_parser(["1", "2"]),
        )
        .arg(clap::Arg::new("root-pos").value_parser(["input", "output"]))
        .subcommand(
            Command::new("run")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .arg(
                    clap::Arg::new("tag")
                        .long("tag")
                        .num_args(1..=3)
                        .value_parser(["red", "blue", "green"]),
                )
                .arg(clap::Arg::new("run-pos").value_parser(["job", "log"])),
        )
}

fn complete_multi_value_tag(args: &[&str]) -> Vec<CompletionCandidate> {
    let mut cmd = multi_value_tag_cmd();
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(&mut cmd, args, arg_index, None).unwrap()
}

fn multi_value_tag_values(args: &[&str]) -> Vec<String> {
    complete_multi_value_tag(args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

type CandidateMetadata = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<usize>,
    bool,
);

fn candidate_metadata(candidates: Vec<CompletionCandidate>) -> Vec<CandidateMetadata> {
    candidates
        .into_iter()
        .map(|candidate| {
            (
                candidate.get_value().to_string_lossy().into_owned(),
                candidate.get_help().map(|help| help.to_string()),
                candidate.get_id().cloned(),
                candidate.get_tag().map(|tag| tag.to_string()),
                candidate.get_display_order(),
                candidate.is_hide_set(),
            )
        })
        .collect()
}

#[test]
fn multi_value_tag_completes_only_remaining_values() {
    // Until the maximum is reached every plain word is another `--tag` value:
    // possible values can repeat, and no option, subcommand, or positional
    // candidate may leak into the suggestions.
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", "blue", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", "blue", "green", ""]),
        ["run", "input", "output", "--tag", "--limit"]
    );
}

#[test]
fn multi_value_tag_prefix_filters_values() {
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "r"]),
        ["red"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", "g"]),
        ["green"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "x"]),
        Vec::<String>::new()
    );
}

#[test]
fn multi_value_tag_equals_matches_space_state() {
    // An attached value counts toward the option's values, so completing the
    // empty word after `--tag=red` must be identical to completing after
    // `--tag red`, including every piece of candidate metadata.
    let spaced = candidate_metadata(complete_multi_value_tag(&["tool", "--tag", "red", ""]));
    let attached = candidate_metadata(complete_multi_value_tag(&["tool", "--tag=red", ""]));
    assert_eq!(spaced, attached);
    assert_eq!(
        attached.iter().map(|c| c.0.clone()).collect::<Vec<_>>(),
        ["red", "blue", "green"]
    );

    let spaced = candidate_metadata(complete_multi_value_tag(&[
        "tool", "--tag", "red", "blue", "",
    ]));
    let attached = candidate_metadata(complete_multi_value_tag(&[
        "tool", "--tag=red", "--tag", "blue", "",
    ]));
    assert_eq!(spaced, attached);

    // The attached-value completion itself keeps its `--tag=` prefix.
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag=r"]),
        ["--tag=red"]
    );
}

#[test]
fn multi_value_tag_switches_to_other_option() {
    // `--limit` is a new option, not another tag value: the pending tag must
    // end as soon as a flag-like word appears once the minimum is satisfied.
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", "--limit", ""]),
        ["1", "2"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag=red", "--limit", ""]),
        ["1", "2"]
    );

    // A flag-like word while the minimum is still pending cannot be a value
    // and the option is not yet satisfied, so completion is empty rather
    // than leaking options (same as a single-value pending option).
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "--l"]),
        Vec::<String>::new()
    );

    // Once the minimum is met, a flag-like word switches instead: the
    // matching option is offered.
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", "--l"]),
        ["--limit"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag=red", "--l"]),
        ["--limit"]
    );

    // `-` is stdio, not a flag, so it is consumed as an option value (no
    // candidate); the literal `--` is a switch point like it is for
    // multi-value positionals, so the remaining options are offered.
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", "-"]),
        Vec::<String>::new()
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", "--"]),
        ["--tag", "--limit"]
    );
}

#[test]
fn multi_value_tag_terminator_completes_positionals() {
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", "--", ""]),
        ["input", "output"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag=red", "--", ""]),
        ["input", "output"]
    );

    // After the terminator, flag-like text, the subcommand name, unknown
    // words, and another `--` are positional values: never re-enter option
    // parsing, never descend into `run`, and stay successful.
    for word in ["--limit", "--tag", "--", "-x", "run", "unknown"] {
        assert!(
            multi_value_tag_values(&["tool", "--tag", "red", "--", word]).is_empty(),
            "unexpected candidates for {word:?}"
        );
    }

    // Prefix matching still applies to the root positional values.
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "red", "--", "o"]),
        ["output"]
    );
}

#[test]
fn multi_value_tag_works_inside_subcommand() {
    assert_eq!(
        multi_value_tag_values(&["tool", "run", "--tag", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "run", "--tag", "red", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "run", "--tag=red", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "run", "--tag", "red", "blue", "green", ""]),
        ["job", "log", "--tag"]
    );

    // Stays at the `run` level after the terminator.
    assert_eq!(
        multi_value_tag_values(&["tool", "run", "--tag", "red", "--", ""]),
        ["job", "log"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "run", "--tag=red", "--", "j"]),
        ["job"]
    );
    for word in ["--tag", "--", "run", "bogus"] {
        assert!(multi_value_tag_values(&["tool", "run", "--tag", "red", "--", word]).is_empty());
    }
}

#[test]
fn multi_value_tag_invalid_prior_values_still_complete() {
    // A value the parser would reject must not abort completion or wipe out
    // unrelated positional candidates later on the line.
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "bogus", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag=bogus", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "bogus", "--limit", ""]),
        ["1", "2"]
    );
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag", "bogus", "--", ""]),
        ["input", "output"]
    );

    // A prefix with no possible-value match is a successful empty result.
    assert_eq!(
        multi_value_tag_values(&["tool", "--tag=bogus", "zzz"]),
        Vec::<String>::new()
    );
}

#[test]
fn multi_value_tag_metadata_matches_across_forms_and_levels() {
    // The retained tag candidates must carry identical text, help, id, tag,
    // order, and hidden state regardless of space- or equals-separated input,
    // at both the root and the subcommand level.
    let cases: &[(&[&str], &[&str])] = &[
        (
            &["tool", "--tag", "red", "blue", ""],
            &["tool", "--tag=red", "--tag", "blue", ""],
        ),
        (
            &["tool", "run", "--tag", "red", ""],
            &["tool", "run", "--tag=red", ""],
        ),
    ];
    for (spaced, attached) in cases {
        assert_eq!(
            candidate_metadata(complete_multi_value_tag(spaced)),
            candidate_metadata(complete_multi_value_tag(attached)),
            "{spaced:?} vs {attached:?}"
        );
    }
}

fn optional_value_tool_cmd() -> Command {
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(
            clap::Arg::new("color")
                .long("color")
                .num_args(0..=1)
                .value_parser(["auto", "always", "never"])
                .conflicts_with("quiet"),
        )
        .arg(
            clap::Arg::new("mode")
                .long("mode")
                .value_parser(["fast", "slow"]),
        )
        .arg(
            clap::Arg::new("quiet")
                .long("quiet")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(clap::Arg::new("root-pos").value_parser(["src", "dst"]))
        .subcommand(
            Command::new("run")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .arg(
                    clap::Arg::new("color")
                        .long("color")
                        .num_args(0..=1)
                        .value_parser(["auto", "always", "never"]),
                )
                .arg(clap::Arg::new("run-pos").value_parser(["job", "log"])),
        )
}

fn complete_optional_value_tool(args: &[&str]) -> Vec<CompletionCandidate> {
    let mut cmd = optional_value_tool_cmd();
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(&mut cmd, args, arg_index, None).unwrap()
}

fn optional_value_tool_values(args: &[&str]) -> Vec<String> {
    complete_optional_value_tool(args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn pending_optional_value_completes_only_its_values() {
    // `--color` may stand alone, but while it is pending a plain word is its
    // value, so only colors are offered.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", ""]),
        ["auto", "always", "never"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "run", "--color", ""]),
        ["auto", "always", "never"]
    );

    // Prefix filtering still applies within the pending value.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "a"]),
        ["auto", "always"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "run", "--color", "n"]),
        ["never"]
    );
}

#[test]
fn pending_optional_value_switches_to_next_option() {
    // A flag-like word cannot be the optional value: `--color` is treated as
    // not provided and the new option's values are completed.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "--mode", ""]),
        ["fast", "slow"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "--mode", "f"]),
        ["fast"]
    );

    // At the `run` level the pending `--color` likewise stands alone when the
    // next word is flag-like; only `run`-level candidates are offered.
    assert_eq!(
        optional_value_tool_values(&["tool", "run", "--color", "--c"]),
        ["--color"]
    );

    // A flag-like prefix completes options, not color values.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "--m"]),
        ["--mode"]
    );
    // The pending `--color` still counts as present, so the conflicting
    // `--quiet` is filtered out.
    assert!(optional_value_tool_values(&["tool", "--color", "--q"]).is_empty());
}

#[test]
fn optional_value_equals_form_is_preserved() {
    assert_eq!(
        optional_value_tool_values(&["tool", "--color="]),
        ["--color=auto", "--color=always", "--color=never"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "--color=a"]),
        ["--color=auto", "--color=always"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "run", "--color="]),
        ["--color=auto", "--color=always", "--color=never"]
    );
}

#[test]
fn optional_value_option_can_repeat() {
    // After a complete color, another `--color` can still take a value.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "auto", "--color", ""]),
        ["auto", "always", "never"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "--color=auto", "--color", ""]),
        ["auto", "always", "never"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "auto", "--color="]),
        ["--color=auto", "--color=always", "--color=never"]
    );

    // A pending `--color` followed by another `--color`: the first one stands
    // alone and the second one is completed.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "--color", ""]),
        ["auto", "always", "never"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "run", "--color", "always", "--color", ""]),
        ["auto", "always", "never"]
    );
}

#[test]
fn pending_optional_value_terminated_by_escape() {
    // `--` cancels the pending optional value; only the current level's
    // positionals are offered from then on.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "--", ""]),
        ["src", "dst"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "run", "--color", "--", ""]),
        ["job", "log"]
    );

    // The `run` level must not fall back to root positionals.
    assert!(optional_value_tool_values(&["tool", "run", "--color", "--", "src"]).is_empty());

    // Option-style words, the subcommand name, and unknown words after `--`
    // are positional values that match nothing: successful empty results.
    for word in ["--color", "--mode", "-x", "run", "bogus"] {
        assert!(optional_value_tool_values(&["tool", "--color", "--", word]).is_empty());
        assert!(optional_value_tool_values(&["tool", "run", "--color", "--", word]).is_empty());
    }

    // Prefix filtering of the current level's positionals still applies.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "--", "s"]),
        ["src"]
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "run", "--color", "--", "j"]),
        ["job"]
    );
}

#[test]
fn optional_value_conflicts_filter_candidates() {
    // `--quiet` is present: the conflicting `--color` is excluded while
    // unrelated candidates remain.
    assert_eq!(
        optional_value_tool_values(&["tool", "--quiet", ""]),
        ["run", "src", "dst", "--mode", "--quiet"]
    );

    // A completed `--color` excludes `--quiet`.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "auto", ""]),
        ["run", "src", "dst", "--color", "--mode"]
    );

    // A conflicting pair already on the command line must not error or clear
    // unrelated candidates.
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "auto", "--quiet", ""]),
        ["run", "src", "dst", "--mode"]
    );

    // A pending `--color` still completes its values despite the conflict.
    assert_eq!(
        optional_value_tool_values(&["tool", "--quiet", "--color", ""]),
        ["auto", "always", "never"]
    );
}

#[test]
fn optional_value_invalid_prior_words_still_complete() {
    let all = ["run", "src", "dst", "--color", "--mode", "--quiet"];

    // Illegal values, attached or separate, must not error or clear
    // unrelated candidates.
    assert_eq!(
        optional_value_tool_values(&["tool", "--mode", "bogus", ""]),
        all
    );
    // An invalid `--color` value still counts as `--color` being used, so the
    // conflicting `--quiet` is filtered while everything else remains.
    let without_quiet = ["run", "src", "dst", "--color", "--mode"];
    assert_eq!(
        optional_value_tool_values(&["tool", "--color", "bogus", ""]),
        without_quiet
    );
    assert_eq!(
        optional_value_tool_values(&["tool", "--color=bogus", ""]),
        without_quiet
    );

    // Unknown options on the line must not break completion either.
    assert_eq!(optional_value_tool_values(&["tool", "--bogus", ""]), all);
    assert_eq!(
        optional_value_tool_values(&["tool", "--bogus", "--color", ""]),
        ["auto", "always", "never"]
    );

    // Positional slots exhausted: a successful empty result, not an error or
    // a fallback to another level.
    assert!(optional_value_tool_values(&["tool", "--", "src", ""]).is_empty());
    assert!(optional_value_tool_values(&["tool", "run", "--", "job", ""]).is_empty());

    // A prefix with no match is a successful empty result.
    assert!(optional_value_tool_values(&["tool", "--color", "x"]).is_empty());
    assert!(optional_value_tool_values(&["tool", "--color=x"]).is_empty());
    assert!(optional_value_tool_values(&["tool", "run", "--color", "x"]).is_empty());
}

#[test]
fn optional_value_preserves_candidate_metadata() {
    // Value, help, id, tag, display order, and hidden state of the retained
    // candidates must be identical regardless of how earlier words were
    // written or which level they were on.
    type CandidateMetadata = (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<usize>,
        bool,
    );
    fn metadata(candidates: Vec<CompletionCandidate>) -> Vec<CandidateMetadata> {
        candidates
            .into_iter()
            .map(|candidate| {
                (
                    candidate.get_value().to_string_lossy().into_owned(),
                    candidate.get_help().map(|help| help.to_string()),
                    candidate.get_id().cloned(),
                    candidate.get_tag().map(|tag| tag.to_string()),
                    candidate.get_display_order(),
                    candidate.is_hide_set(),
                )
            })
            .collect()
    }

    // Space-separated and equals-separated prior occurrences complete the
    // same pending `--color`.
    let spaced = metadata(complete_optional_value_tool(&[
        "tool", "--color", "auto", "--color", "",
    ]));
    let attached = metadata(complete_optional_value_tool(&[
        "tool", "--color=auto", "--color", "",
    ]));
    assert_eq!(spaced, attached);

    // The nested `run` level defines the same `--color`; its candidates carry
    // the same metadata as the root level's.
    let root = metadata(complete_optional_value_tool(&["tool", "--color", ""]));
    let nested = metadata(complete_optional_value_tool(&["tool", "run", "--color", ""]));
    assert_eq!(root, nested);

    // The equals form only adds the `--color=` prefix to each value; every
    // other piece of metadata is untouched.
    let equals = metadata(complete_optional_value_tool(&["tool", "--color="]));
    assert_eq!(root.len(), equals.len());
    for (spaced, equals) in root.iter().zip(equals.iter()) {
        assert_eq!(format!("--color={}", spaced.0), equals.0);
        assert_eq!(spaced.1, equals.1);
        assert_eq!(spaced.2, equals.2);
        assert_eq!(spaced.3, equals.3);
        assert_eq!(spaced.4, equals.4);
        assert_eq!(spaced.5, equals.5);
    }

    // Cancelling the pending value with `--` may only shrink the candidate
    // set; the retained positionals are byte-for-byte identical.
    let plain = metadata(complete_optional_value_tool(&["tool", ""]));
    let escaped = metadata(complete_optional_value_tool(&["tool", "--color", "--", ""]));
    let expected: Vec<_> = plain
        .iter()
        .filter(|candidate| candidate.0 == "src" || candidate.0 == "dst")
        .cloned()
        .collect();
    assert_eq!(escaped, expected);

    let plain = metadata(complete_optional_value_tool(&["tool", "run", ""]));
    let escaped = metadata(complete_optional_value_tool(&[
        "tool", "run", "--color", "--", "",
    ]));
    let expected: Vec<_> = plain
        .iter()
        .filter(|candidate| candidate.0 == "job" || candidate.0 == "log")
        .cloned()
        .collect();
    assert_eq!(escaped, expected);
}

fn nested_color_tool_cmd() -> Command {
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(
            clap::Arg::new("color")
                .long("color")
                .num_args(0..=1)
                .value_parser(["auto", "always", "never"]),
        )
        .arg(
            clap::Arg::new("quiet")
                .long("quiet")
                .action(clap::ArgAction::SetTrue)
                .conflicts_with("color"),
        )
        .arg(clap::Arg::new("root-pos").value_parser(["src", "dst"]))
        .subcommand(
            Command::new("run")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .arg(
                    clap::Arg::new("color")
                        .long("color")
                        .num_args(0..=1)
                        .value_parser(["auto", "always", "never"]),
                )
                .arg(clap::Arg::new("run-pos").value_parser(["job", "log"])),
        )
}

fn complete_nested_color_tool(args: &[&str]) -> Vec<CompletionCandidate> {
    let mut cmd = nested_color_tool_cmd();
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(&mut cmd, args, arg_index, None).unwrap()
}

fn nested_color_tool_values(args: &[&str]) -> Vec<String> {
    complete_nested_color_tool(args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn root_color_does_not_leak_into_run_pending_color() {
    // A completed root `--color` before `run` must not leak root positionals
    // or the conflicting `--quiet` into the run-level pending `--color`.
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "auto", "run", "--color", ""]),
        ["auto", "always", "never"]
    );
    assert_eq!(
        nested_color_tool_values(&["tool", "--color=auto", "run", "--color", ""]),
        ["auto", "always", "never"]
    );
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "auto", "run", "--color", "a"]),
        ["auto", "always"]
    );
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "auto", "run", "--color="]),
        ["--color=auto", "--color=always", "--color=never"]
    );

    // Root options must not leak into the run level either: only the
    // run-level `--color` matches, never the root-level `--quiet`.
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "auto", "run", "--"]),
        ["--color"]
    );
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "auto", "run", ""]),
        ["job", "log", "--color"]
    );
}

#[test]
fn run_level_color_can_repeat_after_value() {
    // After a complete run-level color, another `--color` can still take a
    // value, whether the earlier value was space- or equals-separated.
    assert_eq!(
        nested_color_tool_values(&["tool", "run", "--color", "auto", "--color", ""]),
        ["auto", "always", "never"]
    );
    assert_eq!(
        nested_color_tool_values(&["tool", "run", "--color=auto", "--color", ""]),
        ["auto", "always", "never"]
    );
    assert_eq!(
        nested_color_tool_values(&["tool", "run", "--color", "auto", "--color="]),
        ["--color=auto", "--color=always", "--color=never"]
    );

    // A root-level color followed by two run-level colors: each level tracks
    // its own occurrences.
    assert_eq!(
        nested_color_tool_values(&[
            "tool", "--color", "never", "run", "--color", "auto", "--color", ""
        ]),
        ["auto", "always", "never"]
    );
}

#[test]
fn pending_color_ends_at_flag_like_word_per_level() {
    // `--quiet` ends the pending value; `--color` stands alone and still
    // counts as present, so the conflicting `--quiet` is filtered out.
    assert!(nested_color_tool_values(&["tool", "--color", "--q"]).is_empty());
    // Once `--quiet` is on the line the conflicting pair excludes both
    // options; the remaining root candidates are still offered.
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "--quiet", ""]),
        ["run", "src", "dst"]
    );

    // An unknown long prefix ends the pending value with a successful empty
    // result, and completion afterwards reflects only the current level.
    assert!(nested_color_tool_values(&["tool", "--color", "--bogus"]).is_empty());
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "--bogus", ""]),
        ["run", "src", "dst", "--color"]
    );

    // Same at the run level: only run-level candidates, no parent state.
    assert_eq!(
        nested_color_tool_values(&["tool", "run", "--color", "--c"]),
        ["--color"]
    );
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "auto", "run", "--color", "--c"]),
        ["--color"]
    );
    // The root-level `--quiet` does not exist at the run level.
    assert!(nested_color_tool_values(&["tool", "run", "--color", "--q"]).is_empty());
    assert_eq!(
        nested_color_tool_values(&["tool", "run", "--color", "--bogus", ""]),
        ["job", "log", "--color"]
    );
}

#[test]
fn terminator_after_pending_color_completes_own_level_positionals() {
    // `--` cancels the pending optional value; only the current level's
    // positionals are offered from then on.
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "--", ""]),
        ["src", "dst"]
    );
    assert_eq!(
        nested_color_tool_values(&["tool", "run", "--color", "--", ""]),
        ["job", "log"]
    );

    // A root-level color before `run` does not change which level the
    // terminator belongs to.
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "auto", "run", "--color", "--", ""]),
        ["job", "log"]
    );

    // The run level must not fall back to root positionals.
    assert!(nested_color_tool_values(&["tool", "run", "--color", "--", "src"]).is_empty());
    assert_eq!(
        nested_color_tool_values(&["tool", "run", "--color", "--", "j"]),
        ["job"]
    );
    assert_eq!(
        nested_color_tool_values(&["tool", "--color", "--", "s"]),
        ["src"]
    );
}

#[test]
fn after_terminator_words_stay_positional_per_level() {
    // Subcommand names, option-style words, and unknown words after `--` are
    // positional values that match nothing: successful empty results, with no
    // level switching and no fallback to the parent level.
    for word in ["run", "--color", "--quiet", "-x", "bogus"] {
        assert!(
            nested_color_tool_values(&["tool", "--color", "--", word]).is_empty(),
            "unexpected candidates for {word:?}"
        );
        assert!(
            nested_color_tool_values(&["tool", "run", "--color", "--", word]).is_empty(),
            "unexpected candidates for {word:?}"
        );
    }

    // A consumed positional slot cannot be completed again and cannot
    // re-enter option parsing or descend into `run`.
    assert!(nested_color_tool_values(&["tool", "--color", "--", "src", ""]).is_empty());
    assert!(nested_color_tool_values(&["tool", "run", "--color", "--", "job", ""]).is_empty());
}

#[test]
fn nested_color_preserves_candidate_metadata() {
    // Value, help, id, tag, display order, and hidden state of the retained
    // candidates must be identical regardless of how earlier words were
    // written or which level they were on.
    let spaced = candidate_metadata(complete_nested_color_tool(&[
        "tool", "--color", "auto", "run", "--color", "",
    ]));
    let attached = candidate_metadata(complete_nested_color_tool(&[
        "tool", "--color=auto", "run", "--color", "",
    ]));
    assert_eq!(spaced, attached);

    // The nested `run` level defines the same `--color`; its candidates carry
    // the same metadata as the root level's.
    let root = candidate_metadata(complete_nested_color_tool(&["tool", "--color", ""]));
    let nested = candidate_metadata(complete_nested_color_tool(&["tool", "run", "--color", ""]));
    assert_eq!(root, nested);

    // The equals form only adds the `--color=` prefix to each value; every
    // other piece of metadata is untouched.
    let equals = candidate_metadata(complete_nested_color_tool(&["tool", "run", "--color="]));
    assert_eq!(nested.len(), equals.len());
    for (spaced, equals) in nested.iter().zip(equals.iter()) {
        assert_eq!(format!("--color={}", spaced.0), equals.0);
        assert_eq!(spaced.1, equals.1);
        assert_eq!(spaced.2, equals.2);
        assert_eq!(spaced.3, equals.3);
        assert_eq!(spaced.4, equals.4);
        assert_eq!(spaced.5, equals.5);
    }

    // Cancelling the pending value with `--` may only shrink the candidate
    // set; the retained positionals are byte-for-byte identical.
    let plain = candidate_metadata(complete_nested_color_tool(&["tool", ""]));
    let escaped = candidate_metadata(complete_nested_color_tool(&["tool", "--color", "--", ""]));
    let expected: Vec<_> = plain
        .iter()
        .filter(|candidate| candidate.0 == "src" || candidate.0 == "dst")
        .cloned()
        .collect();
    assert_eq!(escaped, expected);

    let plain = candidate_metadata(complete_nested_color_tool(&["tool", "run", ""]));
    let escaped = candidate_metadata(complete_nested_color_tool(&[
        "tool", "run", "--color", "--", "",
    ]));
    let expected: Vec<_> = plain
        .iter()
        .filter(|candidate| candidate.0 == "job" || candidate.0 == "log")
        .cloned()
        .collect();
    assert_eq!(escaped, expected);
}

fn complete(cmd: &mut Command, args: impl AsRef<str>, current_dir: Option<&Path>) -> String {
    let input = args.as_ref();
    let mut args = vec![std::ffi::OsString::from(cmd.get_name())];
    let arg_index;

    if let Some((prior, after)) = input.split_once("[TAB]") {
        args.extend(prior.split_whitespace().map(From::from));
        if prior.ends_with(char::is_whitespace) {
            args.push(std::ffi::OsString::default());
        }
        arg_index = args.len() - 1;
        // HACK: this cannot handle in-word '[TAB]'
        args.extend(after.split_whitespace().map(From::from));
    } else {
        args.extend(input.split_whitespace().map(From::from));
        if input.ends_with(char::is_whitespace) {
            args.push(std::ffi::OsString::default());
        }
        arg_index = args.len() - 1;
    }

    clap_complete::engine::complete(cmd, args, arg_index, current_dir)
        .unwrap()
        .into_iter()
        .map(|candidate| {
            let compl = candidate.get_value().to_str().unwrap();
            if let Some(help) = candidate.get_help() {
                format!("{compl}\t{help}")
            } else {
                compl.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
