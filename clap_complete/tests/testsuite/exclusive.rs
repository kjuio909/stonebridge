#![cfg(feature = "unstable-dynamic")]

use std::ffi::OsString;

use clap::{ArgGroup, Command};
use clap_complete::engine::CompletionCandidate;

/// The `tool` command exercised throughout this module:
/// - automatic help and version flags (and the help subcommand) are disabled
/// - root: repeatable `--source` (visible alias `--input`) accepting
///   `file`/`stdin`; mutually exclusive `--watch`/`--once`; exclusive `--all`
///   (visible alias `--everything`); subcommand `sync` (visible alias `s`)
/// - `sync`: its own `--source` accepting `remote`/`cache`; mutually exclusive
///   `--force`/`--dry-run`; exclusive `--purge`
fn tool_cmd() -> Command {
    Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(
            clap::Arg::new("source")
                .long("source")
                .visible_alias("input")
                .value_parser(["file", "stdin"])
                .action(clap::ArgAction::Append),
        )
        .group(
            ArgGroup::new("run-mode")
                .args(["watch", "once"])
                .multiple(false),
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
                        .value_parser(["remote", "cache"]),
                )
                .group(
                    ArgGroup::new("sync-mode")
                        .args(["force", "dry-run"])
                        .multiple(false),
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
                .arg(
                    clap::Arg::new("purge")
                        .long("purge")
                        .action(clap::ArgAction::SetTrue)
                        .exclusive(true),
                ),
        )
}

fn os_args(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

/// Run completion with the cursor on the final word.
fn complete_values(args: &[&str]) -> Vec<String> {
    let mut cmd = tool_cmd();
    let args = os_args(args);
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(&mut cmd, args, arg_index, None)
        .unwrap()
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

/// Run completion against an explicit command and cursor.
fn try_complete(
    cmd: &mut Command,
    args: &[&str],
    arg_index: usize,
) -> std::io::Result<Vec<CompletionCandidate>> {
    clap_complete::engine::complete(cmd, os_args(args), arg_index, None)
}

fn complete_values_with(cmd: &mut Command, args: &[&str]) -> Vec<String> {
    let arg_index = args.len() - 1;
    try_complete(cmd, args, arg_index)
        .unwrap()
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

fn values(candidates: &[CompletionCandidate]) -> Vec<String> {
    candidates
        .iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

const ROOT_SCOPE: &[&str] = &[
    "sync",
    "s",
    "--source",
    "--input",
    "--watch",
    "--once",
    "--all",
    "--everything",
];

const SYNC_SCOPE: &[&str] = &["--source", "--force", "--dry-run", "--purge"];

#[test]
fn empty_root_word_lists_everything_still_available() {
    assert_eq!(complete_values(&["tool", ""]), ROOT_SCOPE);
}

#[test]
fn pending_root_source_offers_only_its_values_with_prefix() {
    // Space-separated, main name and visible alias.
    assert_eq!(complete_values(&["tool", "--source", ""]), ["file", "stdin"]);
    assert_eq!(complete_values(&["tool", "--input", ""]), ["file", "stdin"]);
    // A typed prefix is retained.
    assert_eq!(complete_values(&["tool", "--source", "f"]), ["file"]);
    assert_eq!(complete_values(&["tool", "--input", "s"]), ["stdin"]);

    // Attached `=` forms offer the same underlying values, re-prefixed with
    // the flag spelling.
    assert_eq!(
        complete_values(&["tool", "--source="]),
        ["--source=file", "--source=stdin"]
    );
    assert_eq!(
        complete_values(&["tool", "--input="]),
        ["--input=file", "--input=stdin"]
    );
    assert_eq!(complete_values(&["tool", "--source=f"]), ["--source=file"]);
    assert_eq!(complete_values(&["tool", "--input=s"]), ["--input=stdin"]);

    // An attached value in one word and a pending empty word in the next keep
    // the option pending only while the attached value itself is empty; once a
    // value is attached the option is complete and the next word resumes the
    // root scope, exactly like the space-separated form.
    assert_eq!(complete_values(&["tool", "--source=", ""]), ["file", "stdin"]);
    assert_eq!(complete_values(&["tool", "--input=", ""]), ["file", "stdin"]);
    assert_eq!(complete_values(&["tool", "--source=f", ""]), ROOT_SCOPE);
    assert_eq!(complete_values(&["tool", "--input=s", ""]), ROOT_SCOPE);
}

#[test]
fn root_scope_resumes_after_a_completed_source_value() {
    assert_eq!(
        complete_values(&["tool", "--source", "file", ""]),
        ROOT_SCOPE
    );
    assert_eq!(
        complete_values(&["tool", "--input", "stdin", ""]),
        ROOT_SCOPE
    );
    assert_eq!(complete_values(&["tool", "--source=file", ""]), ROOT_SCOPE);
    assert_eq!(complete_values(&["tool", "--input=stdin", ""]), ROOT_SCOPE);
    // `--source` is repeatable, so it can still take another value.
    assert_eq!(
        complete_values(&["tool", "--source", "file", "--source", ""]),
        ["file", "stdin"]
    );
}

#[test]
fn watch_and_once_are_mutually_exclusive() {
    // Consuming one hides both group members (and every alias spelling);
    // unrelated options and the subcommand remain available.
    for consumed in ["--watch", "--once"] {
        let candidates = complete_values(&["tool", consumed, ""]);
        assert!(!candidates.contains(&"--watch".to_owned()));
        assert!(!candidates.contains(&"--once".to_owned()));
        assert_eq!(
            candidates,
            ["sync", "s", "--source", "--input", "--all", "--everything"],
            "{consumed} must hide the whole run-mode group"
        );
    }

    // Prefix completion of the forbidden member succeeds with nothing instead
    // of suggesting a losing combination, in both directions.
    assert!(complete_values(&["tool", "--watch", "--o"]).is_empty());
    assert!(complete_values(&["tool", "--once", "--w"]).is_empty());
    assert!(complete_values(&["tool", "--watch", "--once"]).is_empty());
    assert!(complete_values(&["tool", "--once", "--watch"]).is_empty());

    // A conflicting pair already on the line still completes successfully and
    // offers everything outside the group.
    assert_eq!(
        complete_values(&["tool", "--watch", "--once", ""]),
        ["sync", "s", "--source", "--input", "--all", "--everything"]
    );
    // Repeating the same flag keeps the group closed without failing.
    assert_eq!(
        complete_values(&["tool", "--watch", "--watch", ""]),
        ["sync", "s", "--source", "--input", "--all", "--everything"]
    );
}

#[test]
fn all_is_exclusive_in_every_spelling() {
    for consumed in ["--all", "--everything"] {
        // An empty word after the committed flag offers nothing.
        assert!(complete_values(&["tool", consumed, ""]).is_empty());
        // An ordinary word offers nothing: there is no positional to fill and
        // suggesting `sync` would describe a rejected command line.
        assert!(complete_values(&["tool", consumed, "plainword"]).is_empty());
        // A flag-like word, including the exclusive flag itself, offers nothing.
        for word in [
            "--watch",
            "--once",
            "--source",
            "--input",
            "--all",
            "--source=",
            "--source=f",
            "-x",
            "--",
        ] {
            assert!(
                complete_values(&["tool", consumed, word]).is_empty(),
                "{word} after {consumed} must complete empty"
            );
        }
        // A pending option value after the exclusive flag is offered nothing.
        assert!(complete_values(&["tool", consumed, "--source", ""]).is_empty());
        assert!(complete_values(&["tool", consumed, "--source=f"]).is_empty());
        assert!(complete_values(&["tool", consumed, "--input", ""]).is_empty());
        // The subcommand name and its alias, both the word itself and a later
        // empty word, are empty: completion must not descend into `sync`.
        assert!(complete_values(&["tool", consumed, "sync"]).is_empty());
        assert!(complete_values(&["tool", consumed, "s"]).is_empty());
        assert!(complete_values(&["tool", consumed, "sync", ""]).is_empty());
        assert!(complete_values(&["tool", consumed, "s", ""]).is_empty());
        // Repeating the exclusive flag stays empty.
        assert!(complete_values(&["tool", consumed, consumed, ""]).is_empty());
    }

    // Completing the exclusive flag word itself still works before it is
    // committed.
    assert_eq!(complete_values(&["tool", "--al"]), ["--all"]);
    assert_eq!(complete_values(&["tool", "--ev"]), ["--everything"]);
}

#[test]
fn sync_scope_has_its_own_source_values() {
    for entry in ["sync", "s"] {
        assert_eq!(complete_values(&["tool", entry, ""]), SYNC_SCOPE);

        // Pending values come from the sync-level parser only.
        assert_eq!(
            complete_values(&["tool", entry, "--source", ""]),
            ["remote", "cache"]
        );
        assert_eq!(
            complete_values(&["tool", entry, "--source", "r"]),
            ["remote"]
        );
        assert_eq!(
            complete_values(&["tool", entry, "--source", "c"]),
            ["cache"]
        );
        assert_eq!(
            complete_values(&["tool", entry, "--source="]),
            ["--source=remote", "--source=cache"]
        );
        assert_eq!(
            complete_values(&["tool", entry, "--source=r"]),
            ["--source=remote"]
        );

        // Root-level values must never leak into the subcommand.
        assert!(complete_values(&["tool", entry, "--source", "f"]).is_empty());
        assert!(complete_values(&["tool", entry, "--source=f"]).is_empty());
        // The root-only visible alias `--input` is not an option of `sync`:
        // the word itself matches nothing, and a later empty word resumes the
        // plain sync scope without `--input` or root values appearing.
        assert!(complete_values(&["tool", entry, "--input"]).is_empty());
        assert_eq!(
            complete_values(&["tool", entry, "--input", ""]),
            SYNC_SCOPE
        );
    }
}

#[test]
fn sync_force_and_dry_run_are_mutually_exclusive() {
    for consumed in ["--force", "--dry-run"] {
        let candidates = complete_values(&["tool", "sync", consumed, ""]);
        assert!(!candidates.contains(&"--force".to_owned()));
        assert!(!candidates.contains(&"--dry-run".to_owned()));
        assert_eq!(candidates, ["--source", "--purge"]);
    }

    // Prefix completion of the forbidden member is an empty success.
    assert!(complete_values(&["tool", "sync", "--force", "--d"]).is_empty());
    assert!(complete_values(&["tool", "sync", "--dry-run", "--f"]).is_empty());
    // The cursor on the conflicting second flag itself completes empty.
    assert!(complete_values(&["tool", "sync", "--force", "--dry-run"]).is_empty());
    // With both members already on the line, a later empty word offers only
    // what is outside the group.
    assert_eq!(
        complete_values(&["tool", "sync", "--force", "--dry-run", ""]),
        ["--source", "--purge"]
    );

    // The alias entry path shares the same group rules.
    assert_eq!(
        complete_values(&["tool", "s", "--force", ""]),
        ["--source", "--purge"]
    );
    assert!(complete_values(&["tool", "s", "--dry-run", "--f"]).is_empty());
}

#[test]
fn sync_purge_is_exclusive() {
    for entry in ["sync", "s"] {
        // Completing the flag word itself still works before it is committed.
        assert_eq!(complete_values(&["tool", entry, "--pur"]), ["--purge"]);

        // Once committed, every later word completes empty.
        assert!(complete_values(&["tool", entry, "--purge", ""]).is_empty());
        for word in [
            "plain",
            "--force",
            "--dry-run",
            "--source",
            "--purge",
            "--source=",
            "--source=r",
            "-x",
            "--",
        ] {
            assert!(
                complete_values(&["tool", entry, "--purge", word]).is_empty(),
                "{word} after purge must complete empty"
            );
        }
        assert!(
            complete_values(&["tool", entry, "--purge", "--source", ""]).is_empty()
        );
        assert!(complete_values(&["tool", entry, "--purge", "--source=r"]).is_empty());
    }
}

#[test]
fn exclusive_option_that_takes_a_value_keeps_its_own_value() {
    // An exclusive option with a possible value must still complete that value
    // while it is pending, then lock the scope once the value is committed.
    fn exclusive_value_cmd() -> Command {
        Command::new("t")
            .disable_help_flag(true)
            .disable_version_flag(true)
            .disable_help_subcommand(true)
            .arg(
                clap::Arg::new("only")
                    .long("only")
                    .exclusive(true)
                    .value_parser(["a", "b"]),
            )
            .arg(
                clap::Arg::new("other")
                    .long("other")
                    .action(clap::ArgAction::SetTrue),
            )
    }
    let complete = |args: &[&str]| {
        let mut cmd = exclusive_value_cmd();
        let args = os_args(args);
        let arg_index = args.len() - 1;
        clap_complete::engine::complete(&mut cmd, args, arg_index, None)
            .unwrap()
            .into_iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    };

    // While pending, only the option's own values are offered.
    assert_eq!(complete(&["t", "--only", ""]), ["a", "b"]);
    assert_eq!(complete(&["t", "--only="]), ["--only=a", "--only=b"]);
    assert_eq!(complete(&["t", "--only", "a"]), ["a"]);
    assert_eq!(complete(&["t", "--only=a"]), ["--only=a"]);
    // A flag-like word while the value is pending is not a value, so it is an
    // empty result rather than another option.
    assert!(complete(&["t", "--only", "--oth"]).is_empty());
    // Once the value is committed, the scope locks exactly like a flag form.
    assert!(complete(&["t", "--only", "a", ""]).is_empty());
    assert!(complete(&["t", "--only=a", ""]).is_empty());
    assert!(complete(&["t", "--only", "a", "--other"]).is_empty());
}

#[test]
fn primary_and_alias_spellings_share_a_scope() {
    // Pending values are identical for the root option's two long spellings,
    // whether the value is space-separated or attached (modulo the flag
    // prefix carried by attached candidates).
    fn underlying(candidates: Vec<String>) -> Vec<String> {
        candidates
            .into_iter()
            .map(|candidate| {
                candidate
                    .strip_prefix("--source=")
                    .or_else(|| candidate.strip_prefix("--input="))
                    .map(str::to_owned)
                    .unwrap_or(candidate)
            })
            .collect()
    }

    for line_space in [
        &["tool", "--source", ""][..],
        &["tool", "--input", ""][..],
    ] {
        assert_eq!(complete_values(line_space), ["file", "stdin"]);
    }
    for line_eq in [
        &["tool", "--source="][..],
        &["tool", "--input="][..],
    ] {
        assert_eq!(underlying(complete_values(line_eq)), ["file", "stdin"]);
    }
    for line_eq in [&["tool", "--source=f"][..], &["tool", "--input=f"][..]] {
        assert_eq!(underlying(complete_values(line_eq)), ["file"]);
    }
    for line_eq in [&["tool", "--source=s"][..], &["tool", "--input=s"][..]] {
        assert_eq!(underlying(complete_values(line_eq)), ["stdin"]);
    }

    // After completion, both spellings restore the same root scope.
    for line in [
        &["tool", "--source", "file", ""][..],
        &["tool", "--input", "file", ""][..],
        &["tool", "--source=file", ""][..],
        &["tool", "--input=file", ""][..],
    ] {
        assert_eq!(complete_values(line), ROOT_SCOPE);
    }

    // The subcommand's `--source` behaves identically through both entries and
    // in both value styles.
    for entry in ["sync", "s"] {
        assert_eq!(
            complete_values(&["tool", entry, "--source", ""]),
            ["remote", "cache"]
        );
        assert_eq!(
            underlying(complete_values(&["tool", entry, "--source="])),
            ["remote", "cache"]
        );
        assert_eq!(
            underlying(complete_values(&["tool", entry, "--source=c"])),
            ["cache"]
        );
    }
}

#[test]
fn levels_do_not_share_source_rules() {
    // A root value does not restrict the sync-level source.
    assert_eq!(
        complete_values(&["tool", "--source", "file", "sync", "--source", ""]),
        ["remote", "cache"]
    );
    assert_eq!(
        complete_values(&["tool", "--input=stdin", "s", "--source=r"]),
        ["--source=remote"]
    );
    // A sync-level value does not leak back to the root.
    assert_eq!(
        complete_values(&["tool", "sync", "--source", "remote", "--source", ""]),
        ["remote", "cache"]
    );
}

#[test]
fn invalid_and_unmatched_inputs_are_successful_empty_results() {
    // Illegal source values never fail; the word itself completes empty and a
    // later word sees the normal scope.
    assert!(complete_values(&["tool", "--source", "bogus"]).is_empty());
    assert!(complete_values(&["tool", "--source=bogus"]).is_empty());
    assert!(complete_values(&["tool", "--input", "bogus"]).is_empty());
    assert_eq!(
        complete_values(&["tool", "--source", "bogus", ""]),
        ROOT_SCOPE
    );
    assert_eq!(
        complete_values(&["tool", "--source=bogus", ""]),
        ROOT_SCOPE
    );

    // Illegal values at the sync level stay scoped to sync's own values.
    assert!(complete_values(&["tool", "sync", "--source", "bogus"]).is_empty());
    assert!(complete_values(&["tool", "sync", "--source=bogus"]).is_empty());
    assert_eq!(
        complete_values(&["tool", "sync", "--source", "bogus", ""]),
        SYNC_SCOPE
    );

    // Prefixes that match nothing.
    assert!(complete_values(&["tool", "--zzz"]).is_empty());
    assert!(complete_values(&["tool", "--source", "zzz"]).is_empty());
    assert!(complete_values(&["tool", "zzz"]).is_empty());
    assert!(complete_values(&["tool", "sync", "--zzz"]).is_empty());
    assert!(complete_values(&["tool", "sync", "zzz"]).is_empty());

    // An extra word after a committed exclusive argument is empty, not an
    // error, in both levels.
    assert!(complete_values(&["tool", "--all", "extra", ""]).is_empty());
    assert!(complete_values(&["tool", "sync", "--purge", "extra", ""]).is_empty());
}

#[test]
fn out_of_bounds_cursors_complete_empty_without_error() {
    // The cursor may address the final word ...
    assert!(try_complete(&mut tool_cmd(), &["tool", ""], 1)
        .unwrap()
        .iter()
        .any(|c| c.get_value() == "--watch"));
    // ... but a cursor at or past the end names nothing to complete.
    assert!(try_complete(&mut tool_cmd(), &["tool", ""], 2).unwrap().is_empty());
    assert!(try_complete(&mut tool_cmd(), &["tool", ""], 5).unwrap().is_empty());
    assert!(try_complete(&mut tool_cmd(), &["tool", "--watch", ""], 3).unwrap().is_empty());
}

#[test]
fn terminator_keeps_only_the_current_level() {
    // `tool` declares no positionals at either level, so after `--` nothing is
    // offered; root options and subcommands certainly do not leak. The cursor
    // always addresses the word after the terminator (an empty word for the
    // first case).
    for word in ["", "sync", "s", "--watch", "--all", "plain", "-x"] {
        let line = vec!["tool", "--", word];
        assert!(
            complete_values(&line).is_empty(),
            "{word:?} after the root terminator must complete empty"
        );
    }
    for entry in ["sync", "s"] {
        for word in ["", "--source", "--force", "--purge", "remote", "plain", "-x"] {
            let line = vec!["tool", entry, "--", word];
            assert!(
                complete_values(&line).is_empty(),
                "{word:?} after `{entry} --` must complete empty"
            );
        }
    }
    // A terminator does not rescue a line already locked by an exclusive arg.
    assert!(complete_values(&["tool", "--all", "--", ""]).is_empty());
    assert!(complete_values(&["tool", "sync", "--purge", "--", ""]).is_empty());
}

#[test]
fn calls_on_one_command_never_share_state() {
    let mut cmd = tool_cmd();

    // Fresh root scope.
    assert_eq!(complete_values_with(&mut cmd, &["tool", ""]), ROOT_SCOPE);
    // Alias and value paths leave no mark.
    assert!(complete_values_with(&mut cmd, &["tool", "--input=bogus"]).is_empty());
    assert!(complete_values_with(&mut cmd, &["tool", "--all", "x"]).is_empty());
    assert!(complete_values_with(&mut cmd, &["tool", "--watch", "--once"]).is_empty());
    // Descending into the subcommand through its alias is independent.
    assert_eq!(
        complete_values_with(&mut cmd, &["tool", "s", "--purge", ""]),
        Vec::<String>::new()
    );
    assert_eq!(
        complete_values_with(&mut cmd, &["tool", "sync", ""]),
        SYNC_SCOPE
    );
    assert_eq!(
        complete_values_with(&mut cmd, &["tool", "sync", "--force", ""]),
        ["--source", "--purge"]
    );
    // Back at the root the command looks brand new.
    assert_eq!(complete_values_with(&mut cmd, &["tool", ""]), ROOT_SCOPE);
    // A root mutex choice cannot leak into a later sync completion.
    assert_eq!(
        complete_values_with(&mut cmd, &["tool", "--watch", "s", ""]),
        SYNC_SCOPE
    );
    // A terminator-locked call cannot leak into a later legal call.
    assert!(complete_values_with(&mut cmd, &["tool", "--all", "--"]).is_empty());
    assert_eq!(complete_values_with(&mut cmd, &["tool", ""]), ROOT_SCOPE);
    assert_eq!(
        complete_values_with(&mut cmd, &["tool", "sync", ""]),
        SYNC_SCOPE
    );
    // The same candidates as a freshly constructed command, including
    // metadata: interleaving failed inputs never degrades later results.
    let fresh_root = values(&try_complete(&mut tool_cmd(), &["tool", ""], 1).unwrap());
    assert_eq!(complete_values_with(&mut cmd, &["tool", ""]), fresh_root);
    let fresh_sync = values(
        &try_complete(&mut tool_cmd(), &["tool", "sync", ""], 2).unwrap(),
    );
    assert_eq!(
        complete_values_with(&mut cmd, &["tool", "sync", ""]),
        fresh_sync
    );
}
