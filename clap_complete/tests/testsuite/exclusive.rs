#![cfg(feature = "unstable-dynamic")]

use clap::{ArgGroup, Command};
use clap_complete::engine::CompletionCandidate;

/// Build the `tool` command shared by these tests:
/// - automatic help and version are disabled
/// - root options:
///   - `--source` (visible alias `--input`), repeatable, one of `file`/`stdin`
///   - mutually exclusive `--watch` and `--once`
///   - exclusive `--all` with visible alias `--everything`
/// - the `sync` subcommand (visible alias `s`) with:
///   - `--source`, one of `remote`/`cache`
///   - mutually exclusive `--force` and `--dry-run`
///   - exclusive `--purge`
///
/// Each mutually-exclusive pair is a non-multiple [`ArgGroup`], so selecting
/// either member closes the whole pair, matching the parser's one-of rule.
fn tool_cmd() -> Command {
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(
            clap::Arg::new("source")
                .long("source")
                .visible_alias("input")
                .action(clap::ArgAction::Append)
                .value_parser(["file", "stdin"]),
        )
        .arg(
            clap::Arg::new("watch")
                .long("watch")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("once")
                .long("once")
                .action(clap::ArgAction::SetTrue),
        )
        .group(ArgGroup::new("mode").args(["watch", "once"]))
        .arg(
            clap::Arg::new("all")
                .long("all")
                .visible_alias("everything")
                .action(clap::ArgAction::SetTrue)
                .exclusive(true),
        )
        .subcommand(
            Command::new("sync")
                .visible_alias("s")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .disable_help_subcommand(true)
                .arg(
                    clap::Arg::new("source")
                        .long("source")
                        .action(clap::ArgAction::Set)
                        .value_parser(["remote", "cache"]),
                )
                .arg(
                    clap::Arg::new("force")
                        .long("force")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::Arg::new("dry-run")
                        .long("dry-run")
                        .action(clap::ArgAction::SetTrue),
                )
                .group(ArgGroup::new("sync-mode").args(["force", "dry-run"]))
                .arg(
                    clap::Arg::new("purge")
                        .long("purge")
                        .action(clap::ArgAction::SetTrue)
                        .exclusive(true),
                ),
        )
}

fn complete(cmd: &mut Command, args: &[&str]) -> Vec<CompletionCandidate> {
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(cmd, args, arg_index, None).unwrap()
}

fn complete_index(args: &[&str], arg_index: usize) -> Vec<CompletionCandidate> {
    let mut cmd = tool_cmd();
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    clap_complete::engine::complete(&mut cmd, args, arg_index, None).unwrap()
}

fn values(args: &[&str]) -> Vec<String> {
    let mut cmd = tool_cmd();
    complete(&mut cmd, args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

fn values_on(cmd: &mut Command, args: &[&str]) -> Vec<String> {
    complete(cmd, args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

const ROOT_SCOPE: [&str; 8] = [
    "sync",
    "s",
    "--source",
    "--input",
    "--watch",
    "--once",
    "--all",
    "--everything",
];

const SYNC_SCOPE: [&str; 4] = ["--source", "--force", "--dry-run", "--purge"];

#[test]
fn empty_word_lists_root_options_and_subcommand() {
    // With no consumed arguments, an empty last word still offers every
    // available option spelling, the subcommand, and its visible alias.
    assert_eq!(values(&["tool", ""]), ROOT_SCOPE);
}

#[test]
fn source_takes_only_its_values_preserving_the_prefix() {
    // Space-separated, under the main name and the visible alias.
    assert_eq!(values(&["tool", "--source", ""]), ["file", "stdin"]);
    assert_eq!(values(&["tool", "--input", ""]), ["file", "stdin"]);
    assert_eq!(values(&["tool", "--source", "f"]), ["file"]);
    assert_eq!(values(&["tool", "--input", "s"]), ["stdin"]);
    assert!(values(&["tool", "--source", "x"]).is_empty());

    // Equals-attached, under both spellings: candidates keep the flag prefix.
    assert_eq!(
        values(&["tool", "--source="]),
        ["--source=file", "--source=stdin"]
    );
    assert_eq!(
        values(&["tool", "--input="]),
        ["--input=file", "--input=stdin"]
    );
    assert_eq!(values(&["tool", "--source=f"]), ["--source=file"]);
    assert_eq!(values(&["tool", "--input=s"]), ["--input=stdin"]);
    assert!(values(&["tool", "--source=x"]).is_empty());
}

#[test]
fn root_scope_resumes_after_source_value() {
    // Once the value is complete, space and equals paths restore the full
    // root scope on the following empty word.
    assert_eq!(values(&["tool", "--source", "file", ""]), ROOT_SCOPE);
    assert_eq!(values(&["tool", "--source=file", ""]), ROOT_SCOPE);
    assert_eq!(values(&["tool", "--input", "stdin", ""]), ROOT_SCOPE);
    assert_eq!(values(&["tool", "--input=stdin", ""]), ROOT_SCOPE);

    // `--source` is repeatable: another value waits on its own possible values.
    assert_eq!(
        values(&["tool", "--source", "file", "--source", ""]),
        ["file", "stdin"]
    );
    assert_eq!(
        values(&["tool", "--source", "file", "--input=s"]),
        ["--input=stdin"]
    );
    assert_eq!(
        values(&["tool", "--source", "file", "--source", "stdin", ""]),
        ROOT_SCOPE
    );
}

#[test]
fn watch_and_once_are_mutually_exclusive() {
    // Selecting either member closes the whole pair: both `--watch` and
    // `--once` disappear for the next word.
    assert_eq!(
        values(&["tool", "--watch", ""]),
        ["sync", "s", "--source", "--input", "--all", "--everything"]
    );
    assert_eq!(
        values(&["tool", "--once", ""]),
        ["sync", "s", "--source", "--input", "--all", "--everything"]
    );

    // Completing the excluded member, or repeating the selected one, is a
    // successful empty result.
    assert!(values(&["tool", "--watch", "--once"]).is_empty());
    assert!(values(&["tool", "--once", "--watch"]).is_empty());
    assert!(values(&["tool", "--watch", "--watch"]).is_empty());
    assert!(values(&["tool", "--once", "--once"]).is_empty());
    assert!(values(&["tool", "--watch", "--on"]).is_empty());
    assert!(values(&["tool", "--once", "--wat"]).is_empty());

    // A command line already carrying the conflicting pair still completes
    // successfully and offers every non-conflicting candidate.
    assert_eq!(
        values(&["tool", "--watch", "--once", ""]),
        ["sync", "s", "--source", "--input", "--all", "--everything"]
    );
}

#[test]
fn all_is_exclusive_once_consumed() {
    for spelling in ["--all", "--everything"] {
        // Empty word, plain word, option-style word, and a subcommand name or
        // alias after the exclusive flag all succeed with an empty set.
        assert!(values(&["tool", spelling, ""]).is_empty(), "{spelling} + empty");
        assert!(values(&["tool", spelling, "word"]).is_empty(), "{spelling} + word");
        assert!(values(&["tool", spelling, "--x"]).is_empty(), "{spelling} + option");
        assert!(
            values(&["tool", spelling, "sync"]).is_empty(),
            "{spelling} + sync"
        );
        assert!(values(&["tool", spelling, "s"]).is_empty(), "{spelling} + s");

        // Descent into the subcommand is blocked, so completion stays empty
        // even after a would-be subcommand word.
        assert!(
            values(&["tool", spelling, "sync", ""]).is_empty(),
            "{spelling} sync empty"
        );
        assert!(
            values(&["tool", spelling, "s", "--source", ""]).is_empty(),
            "{spelling} s --source"
        );
        assert!(
            values(&["tool", spelling, "--", ""]).is_empty(),
            "{spelling} terminator"
        );
    }

    // Completing the exclusive flag spelling itself, before it is committed,
    // still resolves the prefix normally.
    assert_eq!(values(&["tool", "--all"]), ["--all"]);
    assert_eq!(values(&["tool", "--everything"]), ["--everything"]);
    assert_eq!(values(&["tool", "--eve"]), ["--everything"]);
}

#[test]
fn sync_scope_has_its_own_options_and_values() {
    for entry in ["sync", "s"] {
        // No root options, values, or subcommands leak into the scope.
        assert_eq!(values(&["tool", entry, ""]), SYNC_SCOPE);
        for leaked in ["file", "stdin", "--input", "--watch", "--once", "--all"] {
            assert!(
                !values(&["tool", entry, ""]).contains(&leaked.to_owned()),
                "`{leaked}` must not leak into sync scope"
            );
        }

        // The subcommand-level source only knows remote/cache.
        assert_eq!(
            values(&["tool", entry, "--source", ""]),
            ["remote", "cache"]
        );
        assert_eq!(values(&["tool", entry, "--source", "r"]), ["remote"]);
        assert_eq!(
            values(&["tool", entry, "--source="]),
            ["--source=remote", "--source=cache"]
        );
        assert_eq!(
            values(&["tool", entry, "--source=c"]),
            ["--source=cache"]
        );
        assert!(values(&["tool", entry, "--source", "file"]).is_empty());
        assert!(values(&["tool", entry, "--source=file"]).is_empty());
    }
}

#[test]
fn sync_force_and_dry_run_are_mutually_exclusive() {
    for entry in ["sync", "s"] {
        // Selecting either member closes the whole pair.
        assert_eq!(
            values(&["tool", entry, "--force", ""]),
            ["--source", "--purge"]
        );
        assert_eq!(
            values(&["tool", entry, "--dry-run", ""]),
            ["--source", "--purge"]
        );
        assert!(values(&["tool", entry, "--force", "--dry-run"]).is_empty());
        assert!(values(&["tool", entry, "--dry-run", "--force"]).is_empty());
        assert!(values(&["tool", entry, "--force", "--force"]).is_empty());
        assert!(values(&["tool", entry, "--force", "--dry"]).is_empty());
        assert_eq!(
            values(&["tool", entry, "--force", "--dry-run", ""]),
            ["--source", "--purge"]
        );
    }
}

#[test]
fn sync_purge_is_exclusive_once_consumed() {
    for entry in ["sync", "s"] {
        assert!(
            values(&["tool", entry, "--purge", ""]).is_empty(),
            "{entry} purge empty"
        );
        assert!(
            values(&["tool", entry, "--purge", "word"]).is_empty(),
            "{entry} purge word"
        );
        assert!(
            values(&["tool", entry, "--purge", "--force"]).is_empty(),
            "{entry} purge option"
        );
        assert!(
            values(&["tool", entry, "--purge", "--", ""]).is_empty(),
            "{entry} purge terminator"
        );
        // Completing the flag itself still resolves the prefix.
        assert_eq!(
            values(&["tool", entry, "--pur"]),
            ["--purge"]
        );
    }
}

#[test]
fn main_and_alias_space_and_equals_paths_share_scope() {
    // The same available range under the source main name and the root alias,
    // whether the value waits after a space or is attached with `=`.
    assert_eq!(
        values(&["tool", "--source", ""]),
        values(&["tool", "--input", ""])
    );
    assert_eq!(
        values(&["tool", "--source="]),
        values(&["tool", "--input="])
            .into_iter()
            .map(|value| value.replace("--input", "--source"))
            .collect::<Vec<_>>()
    );

    // Subcommand entry by name or alias yields the identical scope, including
    // after the subcommand-level mutex or exclusive choice.
    assert_eq!(
        values(&["tool", "sync", ""]),
        values(&["tool", "s", ""])
    );
    assert_eq!(
        values(&["tool", "sync", "--force", ""]),
        values(&["tool", "s", "--force", ""])
    );
    assert_eq!(
        values(&["tool", "sync", "--source", ""]),
        values(&["tool", "s", "--source", ""])
    );
}

#[test]
fn source_scopes_do_not_cross_levels() {
    // Root-consumed source values cannot change the subcommand source range.
    for prefix in [
        &["tool", "--source", "file"][..],
        &["tool", "--source=stdin"][..],
        &["tool", "--input", "file"][..],
        &["tool", "--input=stdin"][..],
    ] {
        for entry in ["sync", "s"] {
            let mut line = prefix.to_vec();
            line.push(entry);
            line.push("--source");
            line.push("");
            assert_eq!(values(&line), ["remote", "cache"], "{line:?}");
        }
    }

    // A subcommand source choice likewise does not change a later root call;
    // the two are separate calls because the scope only flows downward.
    assert_eq!(values(&["tool", ""]), ROOT_SCOPE);
    assert_eq!(values(&["tool", "sync", "--source", "remote", ""]), SYNC_SCOPE);
    assert_eq!(values(&["tool", ""]), ROOT_SCOPE);
}

#[test]
fn invalid_inputs_succeed_empty_and_do_not_pollute() {
    // Illegal values and unmatched value prefixes are empty successes.
    assert!(values(&["tool", "--source", "bogus"]).is_empty());
    assert!(values(&["tool", "--source=bogus"]).is_empty());
    assert!(values(&["tool", "--input", "bogus"]).is_empty());
    assert!(values(&["tool", "--sync"]).is_empty());
    assert!(values(&["tool", "sync", "--source", "bogus"]).is_empty());
    assert!(values(&["tool", "sync", "--bogus"]).is_empty());

    // After an illegal value the next empty word restores the normal scope.
    assert_eq!(
        values(&["tool", "--source", "bogus", ""]),
        ROOT_SCOPE
    );
    assert_eq!(
        values(&["tool", "--source=bogus", ""]),
        ROOT_SCOPE
    );

    // Out-of-range cursors succeed with an empty set.
    assert!(complete_index(&["tool", ""], 5).is_empty());
    assert!(complete_index(&["tool", "--watch"], 4).is_empty());
}

#[test]
fn terminator_keeps_only_current_level_values() {
    // Neither command defines positionals, so after `--` at either level no
    // option, subcommand, or alias may be offered.
    assert!(values(&["tool", "--", ""]).is_empty());
    assert!(values(&["tool", "sync", "--", ""]).is_empty());
    assert!(values(&["tool", "s", "--", ""]).is_empty());
    for word in ["sync", "s", "--source", "--watch", "--all", "word", "-x"] {
        assert!(values(&["tool", "--", word]).is_empty(), "root `-- {word}`");
    }
    for word in ["--source", "--force", "--purge", "remote", "word", "-x"] {
        assert!(
            values(&["tool", "sync", "--", word]).is_empty(),
            "sync `-- {word}`"
        );
    }
}

#[test]
fn consecutive_calls_on_one_command_are_isolated() {
    // Failed inputs, alias paths, the other level, and the terminator are
    // interleaved on one Command; a later legal call must match a fresh
    // command, with no consumed state or cross-level leakage.
    let mut cmd = tool_cmd();
    assert_eq!(values_on(&mut cmd, &["tool", ""]), ROOT_SCOPE);

    assert!(values_on(&mut cmd, &["tool", "--watch", "--once"]).is_empty());
    assert_eq!(
        values_on(&mut cmd, &["tool", "--watch", ""]),
        ["sync", "s", "--source", "--input", "--all", "--everything"]
    );

    assert!(values_on(&mut cmd, &["tool", "--source=bogus"]).is_empty());
    assert!(values_on(&mut cmd, &["tool", "s", "--purge", "word"]).is_empty());
    assert_eq!(
        values_on(&mut cmd, &["tool", "s", ""]),
        SYNC_SCOPE
    );

    assert!(values_on(&mut cmd, &["tool", "--all", "sync", ""]).is_empty());
    assert!(values_on(&mut cmd, &["tool", "sync", "--force", "--dry-run"]).is_empty());

    assert!(values_on(&mut cmd, &["tool", "--", ""]).is_empty());
    assert!(values_on(&mut cmd, &["tool", "sync", "--", ""]).is_empty());

    // Every legal call now matches a freshly built command.
    let mut fresh = tool_cmd();
    assert_eq!(
        values_on(&mut cmd, &["tool", ""]),
        values_on(&mut fresh, &["tool", ""])
    );
    assert_eq!(values_on(&mut cmd, &["tool", ""]), ROOT_SCOPE);
    let mut fresh = tool_cmd();
    assert_eq!(
        values_on(&mut cmd, &["tool", "--source", ""]),
        values_on(&mut fresh, &["tool", "--source", ""])
    );
    let mut fresh = tool_cmd();
    assert_eq!(
        values_on(&mut cmd, &["tool", "sync", ""]),
        values_on(&mut fresh, &["tool", "sync", ""])
    );
    assert_eq!(values_on(&mut cmd, &["tool", "sync", ""]), SYNC_SCOPE);
    let mut fresh = tool_cmd();
    assert_eq!(
        values_on(&mut cmd, &["tool", "sync", "--source", ""]),
        values_on(&mut fresh, &["tool", "sync", "--source", ""])
    );
    let mut fresh = tool_cmd();
    assert_eq!(
        values_on(&mut cmd, &["tool", "s", "--force", ""]),
        values_on(&mut fresh, &["tool", "s", "--force", ""])
    );
}
