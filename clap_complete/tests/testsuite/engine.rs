#![cfg(feature = "unstable-dynamic")]

use std::fs;
use std::path::Path;

use clap::{builder::PossibleValue, Command};
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

    assert_data_eq!(complete!(cmd, "test"), snapbox::str!["test_visible"]);

    assert_data_eq!(complete!(cmd, "test_h"), snapbox::str!["test_hidden"]);

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
hello-moon
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

    assert_data_eq!(complete!(cmd, "--test"), snapbox::str!["--test_visible"]);

    assert_data_eq!(complete!(cmd, "--test_h"), snapbox::str!["--test_hidden"]);

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
        ["--json", "--toml", "--verbose", "--quiet"]
    );

    // `--json` conflicts with `--toml`; unrelated candidates remain.
    let completions = complete_tool(&["tool", "--json", ""]);
    assert!(completions.contains(&"--verbose".to_owned()));
    assert!(completions.contains(&"--quiet".to_owned()));
    assert!(!completions.contains(&"--toml".to_owned()));
    assert_eq!(completions, ["--json", "--verbose", "--quiet"]);
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
    assert_eq!(completions, ["--json", "--verbose", "--quiet"]);
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
    assert_eq!(completions, ["--json", "--toml"]);
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

fn tag_tool_cmd() -> Command {
    let tag = || {
        clap::Arg::new("tag")
            .long("tag")
            .value_parser(["red", "blue", "green"])
            .num_args(1..=3)
    };
    let limit = || {
        clap::Arg::new("limit")
            .long("limit")
            .value_parser(["1", "2"])
            .num_args(1)
    };
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(tag())
        .arg(limit())
        .arg(clap::Arg::new("input").value_parser(["input", "output"]))
        .subcommand(
            Command::new("run")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .arg(tag())
                .arg(limit())
                .arg(clap::Arg::new("job").value_parser(["job", "log"])),
        )
}

fn complete_tag_tool(args: &[&str]) -> Vec<CompletionCandidate> {
    let mut cmd = tag_tool_cmd();
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(&mut cmd, args, arg_index, None).unwrap()
}

fn tag_tool_values(args: &[&str]) -> Vec<String> {
    complete_tag_tool(args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn pending_multi_value_option_completes_only_its_values() {
    // `--tag` takes one to three values; while it can still consume the word
    // under the cursor, only its values are offered.  Options, subcommands,
    // and positional values must not leak into the candidates.
    assert_eq!(
        tag_tool_values(&["tool", "--tag", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "--tag", "red", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "--tag", "red", "blue", ""]),
        ["red", "blue", "green"]
    );
}

#[test]
fn multi_value_option_equals_form_matches_spaced_form() {
    // `--tag=red` consumes the first value inline; the option can still take
    // more values, so the following word must complete exactly like the
    // spaced form, down to the candidate metadata.
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

    let spaced = metadata(complete_tag_tool(&["tool", "--tag", "red", ""]));
    let equals = metadata(complete_tag_tool(&["tool", "--tag=red", ""]));
    assert_eq!(equals, spaced);
    assert_eq!(
        equals.iter().map(|candidate| &candidate.0).collect::<Vec<_>>(),
        ["red", "blue", "green"]
    );

    assert_eq!(
        tag_tool_values(&["tool", "--tag=red", "blue", ""]),
        ["red", "blue", "green"]
    );
}

#[test]
fn multi_value_option_done_after_max_values() {
    // Once `--tag` has all three values, it is done and the next word
    // completes like any other word at this level.
    let completions = tag_tool_values(&["tool", "--tag", "red", "blue", "green", ""]);
    assert!(completions.contains(&"--tag".to_owned()));
    assert!(completions.contains(&"--limit".to_owned()));
    assert!(completions.contains(&"run".to_owned()));
    assert!(completions.contains(&"input".to_owned()));
    assert!(!completions.contains(&"red".to_owned()));
}

#[test]
fn multi_value_option_switches_to_next_option() {
    // A flag-like word ends `--tag`'s values; `--limit` must not be swallowed
    // as a tag value and completes its own values.
    assert_eq!(
        tag_tool_values(&["tool", "--tag", "red", "--limit", ""]),
        ["1", "2"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "--tag=red", "--limit", ""]),
        ["1", "2"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "--tag", "red", "blue", "green", "--limit", ""]),
        ["1", "2"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "--tag", "red", "--limit", "1", "--tag", ""]),
        ["red", "blue", "green"]
    );
}

#[test]
fn multi_value_option_terminator_completes_positionals() {
    // `--` ends `--tag`'s values and escapes the rest of the command line;
    // only the current level's positional values are offered.
    assert_eq!(
        tag_tool_values(&["tool", "--tag", "red", "--", ""]),
        ["input", "output"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "--tag=red", "--", ""]),
        ["input", "output"]
    );
    // Value parsing is not resumed after the terminator: option-style words,
    // subcommand names, and unknown words are positional values and complete
    // to a successful empty result.
    assert!(tag_tool_values(&["tool", "--tag", "red", "--", "--limit"]).is_empty());
    assert!(tag_tool_values(&["tool", "--tag", "red", "--", "run"]).is_empty());
    assert!(tag_tool_values(&["tool", "--tag", "red", "--", "bogus"]).is_empty());
    // A subcommand name after `--` is a positional value, not a descent into
    // `run`; with the root positional slot filled, nothing is left to offer.
    assert!(tag_tool_values(&["tool", "--tag", "red", "--", "run", ""]).is_empty());
    assert!(tag_tool_values(&["tool", "--tag", "red", "--", "input", ""]).is_empty());
}

#[test]
fn nested_multi_value_option_does_not_fall_back_to_root() {
    // The same behaviors hold inside `run`, scoped to `run`'s own arguments.
    assert_eq!(
        tag_tool_values(&["tool", "run", "--tag", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "run", "--tag", "red", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "run", "--tag=red", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "run", "--tag", "red", "--limit", ""]),
        ["1", "2"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "run", "--tag", "red", "--", ""]),
        ["job", "log"]
    );
    assert!(tag_tool_values(&["tool", "run", "--tag", "red", "--", "--limit"]).is_empty());
    assert!(tag_tool_values(&["tool", "run", "--tag", "red", "--", "job", ""]).is_empty());
}

#[test]
fn invalid_multi_value_option_value_still_completes() {
    // An invalid `--tag` value earlier on the command line is a parse error
    // for `clap` but must not fail completion or clear unrelated positions.
    assert_eq!(
        tag_tool_values(&["tool", "--tag", "purple", ""]),
        ["red", "blue", "green"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "--tag", "purple", "--limit", ""]),
        ["1", "2"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "--tag", "purple", "--", ""]),
        ["input", "output"]
    );
    assert_eq!(
        tag_tool_values(&["tool", "run", "--tag", "purple", "--", ""]),
        ["job", "log"]
    );
}

#[test]
fn multi_value_option_without_prefix_match_is_empty() {
    // No possible value matches the prefix: a successful empty result.
    assert!(tag_tool_values(&["tool", "--tag", "x"]).is_empty());
    assert!(tag_tool_values(&["tool", "--tag", "red", "x"]).is_empty());
    assert!(tag_tool_values(&["tool", "--tag=red", "x"]).is_empty());
    assert!(tag_tool_values(&["tool", "run", "--tag", "red", "x"]).is_empty());
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
--optional-flag
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
--optional-flag
--long-flag
--help	Print help
"#]]
    );
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
