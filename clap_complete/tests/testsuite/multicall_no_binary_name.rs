#![cfg(feature = "unstable-dynamic")]

//! Dynamic completion for a binary that is both a multicall dispatcher and a
//! no-binary-name command.
//!
//! With both settings on, `argv[0]` is still the applet selector: the engine
//! resolves its file stem to a top-level applet (by name or alias), strips
//! directory and extension, and then completes with the no-binary-name cursor
//! semantics, where index 0 is a word the user is still typing.
//!
//! The aggregate command declares two top-level applets:
//! - `busybox` (visible alias `bb`), which groups `status` (visible alias
//!   `st`, hidden alias `legacy-status`) and `stop`
//! - a direct `status` applet
//!
//! Both `busybox` and the direct `status` applet expose the same inheritable
//! `--config` option, so the two paths into status are directly comparable.

use std::ffi::OsString;
use std::io;

use clap::{builder::PossibleValue, Arg, ArgGroup, Command};
use clap_complete::engine::{complete, CompletionCandidate};

fn config_arg() -> Arg {
    Arg::new("config")
        .long("config")
        .global(true)
        .value_parser([
            PossibleValue::new("dev").help("development"),
            PossibleValue::new("prod").help("production"),
        ])
}

/// A single-member, non-multiple group expresses the public contract that
/// `--config` can be supplied at most once: once completed, it is neither
/// accepted twice by the parser nor suggested again.
fn config_group() -> ArgGroup {
    ArgGroup::new("config_group").arg("config")
}

/// The `status` applet body, shared verbatim by the nested and direct applets
/// so every candidate carries identical metadata on both paths.
fn status_cmd() -> Command {
    Command::new("status")
        .about("Show status")
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg(config_arg())
        .group(config_group())
        .arg(
            Arg::new("format")
                .long("format")
                .value_parser([
                    PossibleValue::new("text").help("plain text"),
                    PossibleValue::new("json").help("json output"),
                ]),
        )
}

/// `status` with both aliases, used at every level so the hidden alias can be
/// entered either as a nested subcommand or as the argv0 applet selector.
fn status_with_aliases() -> Command {
    status_cmd().visible_alias("st").alias("legacy-status")
}

fn busybox_cmd() -> Command {
    Command::new("busybox")
        .about("Multicall dispatcher")
        .visible_alias("bb")
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg(config_arg())
        .group(config_group())
        .subcommand(status_with_aliases())
        .subcommand(Command::new("stop").about("Stop the service"))
}

fn aggregate_cmd() -> Command {
    Command::new("busybox")
        .multicall(true)
        .no_binary_name(true)
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .subcommand(busybox_cmd())
        .subcommand(status_with_aliases())
}

/// Complete the last word on a freshly built aggregate command, returning the
/// raw candidates for full-metadata comparisons.
fn complete_fresh(args: &[&str]) -> Vec<CompletionCandidate> {
    let args = args.iter().map(OsString::from).collect::<Vec<_>>();
    let index = args.len() - 1;
    complete(&mut aggregate_cmd(), args, index, None).expect("completion succeeded")
}

fn values(args: &[&str]) -> Vec<String> {
    values_of(&complete_fresh(args))
}

fn values_of(candidates: &[CompletionCandidate]) -> Vec<String> {
    candidates
        .iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

/// Complete an arbitrary word on a reused command.
fn complete_at(
    cmd: &mut Command,
    args: &[&str],
    index: usize,
) -> Result<Vec<CompletionCandidate>, io::Error> {
    let args = args.iter().map(OsString::from).collect::<Vec<_>>();
    complete(cmd, args, index, None)
}

/// Every spelling of the argv0 file name that must resolve to `busybox`.
const BUSYBOX_ARGV0: &[&str] = &[
    "busybox",
    "./busybox",
    "/usr/local/bin/busybox",
    "busybox.exe",
    "/usr/bin/busybox.exe",
    "bb",
    "./bb",
    "/usr/bin/bb",
    "bb.EXE",
];

/// Every spelling of the argv0 file name that must resolve to the `status`
/// applet, including directory components, an extension, and aliases.
const STATUS_ARGV0: &[&str] = &[
    "status",
    "./status",
    "../bin/status",
    "/usr/local/bin/status",
    "status.exe",
    "/usr/bin/status.exe",
    "status.EXE",
    "st",
    "/usr/bin/st",
];

#[test]
fn argv0_spellings_all_resolve_to_busybox() {
    for argv0 in BUSYBOX_ARGV0 {
        assert_eq!(
            values(&[argv0, ""]),
            ["status", "st", "stop", "--config"],
            "argv0 `{argv0}` changed the empty-word candidates"
        );
        assert_eq!(
            values(&[argv0, "st"]),
            ["status", "st", "stop"],
            "argv0 `{argv0}` changed the prefix candidates"
        );
        assert_eq!(
            values(&[argv0, "status", "--format", ""]),
            ["text", "json"],
            "argv0 `{argv0}` changed the nested value candidates"
        );
    }
}

#[test]
fn argv0_spellings_all_resolve_to_status_applet() {
    let nested = complete_fresh(&["/usr/bin/busybox", "status", ""]);
    for argv0 in STATUS_ARGV0 {
        assert_eq!(
            values(&[argv0, ""]),
            values_of(&nested),
            "argv0 `{argv0}` did not match the busybox/status path"
        );
        assert_eq!(
            values(&[argv0, "--format="]),
            values(&["/usr/bin/busybox", "status", "--format="]),
            "argv0 `{argv0}` changed the equals-form candidates"
        );
    }
}

#[test]
fn busybox_empty_word_lists_only_its_scope() {
    assert_eq!(values(&["busybox", ""]), ["status", "st", "stop", "--config"]);
    // The hidden alias and applet internals never appear.
    let listed = values(&["busybox", ""]);
    for leaked in ["legacy-status", "busybox", "--format", "text", "json"] {
        assert!(!listed.contains(&leaked.to_owned()), "leaked `{leaked}`");
    }
}

#[test]
fn status_scope_offers_only_its_options_and_values() {
    for entry in ["status", "st"] {
        let empty = values(&["busybox", entry, ""]);
        assert_eq!(empty, ["--config", "--format"], "entry `{entry}`");
        assert!(!empty.contains(&"stop".to_owned()), "`stop` leaked into status");

        assert_eq!(
            values(&["busybox", entry, "--format", ""]),
            ["text", "json"]
        );
        assert_eq!(
            values(&["busybox", entry, "--format="]),
            ["--format=text", "--format=json"]
        );
        assert_eq!(
            values(&["busybox", entry, "--config", ""]),
            ["dev", "prod"]
        );
        assert_eq!(
            values(&["busybox", entry, "--config="]),
            ["--config=dev", "--config=prod"]
        );
    }
}

#[test]
fn hidden_alias_enters_but_is_never_suggested() {
    // The exact hidden spelling enters the nested status subcommand...
    assert_eq!(
        values(&["busybox", "legacy-status", "--"]),
        ["--config", "--format"]
    );
    // ...and as the argv0 file stem it enters the direct status applet.
    assert_eq!(
        values(&["/usr/bin/legacy-status", "--"]),
        ["--config", "--format"]
    );

    // Neither the empty word nor any prefix suggests it, at either level.
    for word in ["", "l", "le", "legacy", "legacy-status"] {
        let args: Vec<&str> = vec!["busybox", word];
        assert!(
            !values(&args).contains(&"legacy-status".to_owned()),
            "hidden alias suggested for prefix `{word:?}`"
        );
    }
    assert!(
        !values(&["legacy"]).contains(&"legacy-status".to_owned()),
        "hidden alias suggested at the applet selector"
    );
}

#[test]
fn config_keeps_prefix_in_space_and_equals_forms() {
    assert_eq!(values(&["busybox", "--config", "d"]), ["dev"]);
    assert_eq!(values(&["busybox", "--config=p"]), ["--config=prod"]);
    assert_eq!(
        values(&["busybox", "status", "--config", "p"]),
        ["prod"]
    );
    assert_eq!(
        values(&["busybox", "status", "--config=p"]),
        ["--config=prod"]
    );
}

#[test]
fn config_is_not_suggested_again_once_supplied() {
    // At the busybox scope, both space and equals forms drop `--config`; the
    // subcommands remain.
    assert_eq!(
        values(&["busybox", "--config", "dev", ""]),
        ["status", "st", "stop"]
    );
    assert_eq!(values(&["busybox", "--config", "dev", "--"]), [] as [String; 0]);
    assert_eq!(
        values(&["busybox", "--config=prod", ""]),
        ["status", "st", "stop"]
    );
    assert_eq!(values(&["busybox", "--config=prod", "--"]), [] as [String; 0]);

    // The same holds inside status on both entry paths.
    for prefix in [
        vec!["busybox", "status"],
        vec!["busybox", "st"],
        vec!["/usr/bin/status"],
    ] {
        let mut args = prefix.clone();
        args.extend(["--config", "dev", "--"]);
        assert_eq!(values(&args), ["--format"], "path {prefix:?}");
        let mut args = prefix.clone();
        args.extend(["--config=dev", ""]);
        assert_eq!(values(&args), ["--format"], "path {prefix:?}");
    }
}

#[test]
fn nested_and_direct_status_candidates_are_identical() {
    // Full candidate equality: value, order, help, id, tag, display order and
    // the hidden marker must agree between the two routes.
    let cases: &[(&[&str], &[&str])] = &[
        (&["busybox", "status", ""], &["/usr/bin/status", ""]),
        (&["busybox", "st", ""], &["/usr/bin/st", ""]),
        (
            &["busybox", "legacy-status", ""],
            &["/usr/bin/legacy-status", ""],
        ),
        (&["busybox", "status", "--"], &["/usr/bin/status", "--"]),
        (
            &["busybox", "status", "--format", ""],
            &["/usr/bin/status", "--format", ""],
        ),
        (
            &["busybox", "status", "--format="],
            &["/usr/bin/status", "--format="],
        ),
        (
            &["busybox", "status", "--format=t"],
            &["/usr/bin/status", "--format=t"],
        ),
        (
            &["busybox", "status", "--config", ""],
            &["/usr/bin/status", "--config", ""],
        ),
        (
            &["busybox", "status", "--config="],
            &["/usr/bin/status", "--config="],
        ),
        (
            &["busybox", "status", "--config", "dev", "--"],
            &["/usr/bin/status", "--config", "dev", "--"],
        ),
        (
            &["busybox", "status", "--format", "zzz"],
            &["/usr/bin/status", "--format", "zzz"],
        ),
    ];
    for (nested, direct) in cases {
        assert_eq!(
            complete_fresh(nested),
            complete_fresh(direct),
            "candidate metadata differs between {nested:?} and {direct:?}"
        );
    }
}

#[test]
fn unknown_or_miscased_applet_is_an_error_not_root_completion() {
    for argv0 in ["/usr/bin/does-not-exist", "missing.exe", "Status", "STATUS", "Bb"] {
        let err = complete_at(&mut aggregate_cmd(), &[argv0, ""], 1)
            .expect_err("an unrecognized applet reports an error");
        let stem = std::path::Path::new(argv0)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(
            err.to_string().contains(&stem),
            "the error names `{stem}`: {err}"
        );
    }
}

#[test]
fn empty_sets_for_cursors_terminators_illegal_and_unmatched() {
    // A cursor past every word succeeds with nothing.
    assert!(complete_at(&mut aggregate_cmd(), &["/usr/bin/status", ""], 9).unwrap().is_empty());

    // Words after the `--` terminator, and an empty word following them, never
    // error and never surface candidates.
    assert!(complete_at(
        &mut aggregate_cmd(),
        &["/usr/bin/status", "--", "anything"],
        2
    )
    .unwrap()
    .is_empty());
    assert!(complete_at(
        &mut aggregate_cmd(),
        &["/usr/bin/status", "--", "anything", ""],
        3
    )
    .unwrap()
    .is_empty());

    // Illegal values and unmatchable prefixes succeed with empty sets.
    assert!(complete_at(
        &mut aggregate_cmd(),
        &["/usr/bin/status", "--format", "zzz"],
        2
    )
    .unwrap()
    .is_empty());
    assert!(complete_at(&mut aggregate_cmd(), &["busybox", "zzz"], 1)
        .unwrap()
        .is_empty());
    assert!(complete_at(&mut aggregate_cmd(), &["busybox", "legacy"], 1)
        .unwrap()
        .is_empty());
}

#[test]
fn index_zero_completes_the_applet_selector() {
    // With no binary name, index 0 is the word in progress: it completes at the
    // aggregate root instead of being resolved or consumed.
    let mut cmd = aggregate_cmd();
    assert_eq!(values_of(&complete_at(&mut cmd, &[""], 0).unwrap()), [
        "busybox", "bb", "status", "st"
    ]);
    assert_eq!(values_of(&complete_at(&mut cmd, &["s"], 0).unwrap()), [
        "status", "st"
    ]);
    assert_eq!(values_of(&complete_at(&mut cmd, &["bb"], 0).unwrap()), ["bb"]);
    assert!(complete_at(&mut cmd, &["legacy"], 0).unwrap().is_empty());
    // A typed path has no positional home at the argumentless root.
    assert!(complete_at(&mut cmd, &["/usr/bin/st", "x"], 0)
        .unwrap()
        .is_empty());
}

#[test]
fn failed_then_legal_requests_on_same_command_match_fresh() {
    let mut cmd = aggregate_cmd();

    // Unknown applet fails, then the same object completes a legal path with
    // exactly the candidates a freshly built command would produce.
    complete_at(&mut cmd, &["/usr/bin/missing", ""], 1).expect_err("unknown applet");
    assert_eq!(
        complete_at(&mut cmd, &["/usr/bin/status", ""], 1).unwrap(),
        complete_fresh(&["/usr/bin/status", ""])
    );

    // Wrong-case failure likewise leaves no residue.
    complete_at(&mut cmd, &["/usr/bin/Status", ""], 1).expect_err("case mismatch");
    assert_eq!(
        complete_at(&mut cmd, &["busybox", "--config="], 1).unwrap(),
        complete_fresh(&["busybox", "--config="])
    );
}

#[test]
fn hidden_alias_then_visible_requests_match_fresh() {
    let mut cmd = aggregate_cmd();

    // Entering through the hidden alias must not leak into later requests.
    assert_eq!(
        complete_at(&mut cmd, &["busybox", "legacy-status", "--"], 2).unwrap(),
        complete_fresh(&["busybox", "status", "--"])
    );
    assert_eq!(
        complete_at(&mut cmd, &["busybox", ""], 1).unwrap(),
        complete_fresh(&["busybox", ""])
    );
    assert_eq!(
        complete_at(&mut cmd, &["/usr/bin/legacy-status", "--"], 1).unwrap(),
        complete_fresh(&["/usr/bin/status", "--"])
    );
}

#[test]
fn switching_applets_on_same_command_never_leaks() {
    let mut cmd = aggregate_cmd();

    // busybox -> direct status -> busybox/status -> stop -> direct status.
    let checks: &[(&[&str], usize, &[&str])] = &[
        (&["busybox", ""], 1, &["busybox", ""]),
        (&["/usr/bin/status", "--format", ""], 2, &["busybox", "status", "--format", ""]),
        (&["busybox", "status", "--"], 2, &["busybox", "status", "--"]),
        (&["busybox", "stop", ""], 2, &["busybox", "stop", ""]),
        (&["/usr/bin/bb", ""], 1, &["busybox", ""]),
        (&["/usr/bin/status", "--config", ""], 2, &["busybox", "status", "--config", ""]),
        (&["/usr/bin/st", ""], 1, &["busybox", "status", ""]),
    ];
    for (args, index, fresh_args) in checks {
        assert_eq!(
            complete_at(&mut cmd, args, *index).unwrap(),
            complete_fresh(fresh_args),
            "reused command diverged at {args:?}"
        );
    }
}

#[test]
fn used_config_does_not_leak_across_requests() {
    let mut cmd = aggregate_cmd();

    // A completed config in one request is per-request parse state; the next
    // request starts clean.
    assert_eq!(
        values_of(&complete_at(&mut cmd, &["busybox", "--config", "dev", "--"], 3).unwrap()),
        [] as [String; 0]
    );
    assert_eq!(
        values_of(&complete_at(&mut cmd, &["busybox", "--"], 1).unwrap()),
        ["--config"]
    );
    assert_eq!(
        values_of(
            &complete_at(&mut cmd, &["/usr/bin/status", "--config=prod", ""], 2).unwrap()
        ),
        ["--format"]
    );
    assert_eq!(
        values_of(&complete_at(&mut cmd, &["/usr/bin/status", ""], 1).unwrap()),
        ["--config", "--format"]
    );
}
