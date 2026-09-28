#![cfg(feature = "unstable-dynamic")]

use std::ffi::OsString;

use clap::{builder::PossibleValue, Arg, ArgAction, Command};
use clap_complete::engine::CompletionCandidate;

/// The `status` applet, defined identically under the aggregate `busybox`
/// command and as a top-level applet: completion through either entry must
/// produce the exact same candidates.
fn status_cmd() -> Command {
    Command::new("status")
        .about("Show status")
        .arg(
            Arg::new("format")
                .long("format")
                .value_parser([
                    PossibleValue::new("text").help("plain text"),
                    PossibleValue::new("json").help("json output"),
                ]),
        )
}

/// A multicall binary whose aggregate command is named `busybox`:
/// - invoked as `busybox`, it offers the `status` and `stop` subcommands
/// - invoked through a link named after an applet, it runs that applet
fn multicall_cmd() -> Command {
    Command::new("busybox")
        .multicall(true)
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .subcommand(
            Command::new("busybox")
                .disable_help_flag(true)
                .disable_help_subcommand(true)
                .subcommand(status_cmd())
                .subcommand(
                    Command::new("stop")
                        .about("Stop the service")
                        .arg(Arg::new("force").long("force").action(ArgAction::SetTrue)),
                ),
        )
        .subcommand(status_cmd())
}

/// Complete `args`, addressing the last word.
fn complete_str(args: &[&str]) -> Vec<String> {
    complete_str_at(args, args.len() - 1)
}

/// Complete the word at `index` in `args`.
fn complete_str_at(args: &[&str], index: usize) -> Vec<String> {
    complete_raw(
        &mut multicall_cmd(),
        &args.iter().map(OsString::from).collect::<Vec<_>>(),
        index,
    )
    .expect("completion succeeded")
    .into_iter()
    .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
    .collect()
}

/// Complete with a freshly built multicall command, returning the full
/// candidates so help, ids, tags, order and hidden flags can be compared.
fn complete_full(args: &[&str]) -> Vec<CompletionCandidate> {
    let args = args.iter().map(OsString::from).collect::<Vec<_>>();
    let index = args.len() - 1;
    clap_complete::engine::complete(&mut multicall_cmd(), args, index, None)
        .expect("completion succeeded")
}

fn complete_raw(
    cmd: &mut Command,
    args: &[OsString],
    index: usize,
) -> Result<Vec<CompletionCandidate>, std::io::Error> {
    clap_complete::engine::complete(cmd, args.to_vec(), index, None)
}

fn candidate_str(candidates: &[CompletionCandidate]) -> Vec<String> {
    candidates
        .iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn busybox_lists_its_subcommands_at_empty_word() {
    // The aggregate command's empty trailing word offers its own `status` and
    // `stop` subcommands, never the top-level applets or root options.
    assert_eq!(complete_str(&["/usr/bin/busybox", ""]), ["status", "stop"]);
}

#[test]
fn busybox_prefix_filters_subcommands() {
    assert_eq!(complete_str(&["/usr/bin/busybox", "st"]), ["status", "stop"]);
    assert_eq!(complete_str(&["/usr/bin/busybox", "sto"]), ["stop"]);
    assert!(complete_str(&["/usr/bin/busybox", "x"]).is_empty());
}

#[test]
fn busybox_status_completes_values_with_space() {
    assert_eq!(
        complete_str(&["/usr/bin/busybox", "status", "--format", ""]),
        ["text", "json"]
    );
    // The typed prefix is honored.
    assert_eq!(
        complete_str(&["/usr/bin/busybox", "status", "--format", "j"]),
        ["json"]
    );
    assert!(
        complete_str(&["/usr/bin/busybox", "status", "--format", "zzz"]).is_empty(),
        "an impossible value prefix yields a successful empty set"
    );
}

#[test]
fn busybox_status_completes_values_with_equals() {
    assert_eq!(
        complete_str(&["/usr/bin/busybox", "status", "--format="]),
        ["--format=text", "--format=json"]
    );
    assert_eq!(
        complete_str(&["/usr/bin/busybox", "status", "--format=t"]),
        ["--format=text"]
    );
    assert!(
        complete_str(&["/usr/bin/busybox", "status", "--format=zzz"]).is_empty(),
        "an impossible attached prefix yields a successful empty set"
    );
}

#[test]
fn busybox_status_offers_its_option() {
    assert_eq!(
        complete_str(&["/usr/bin/busybox", "status", "--"]),
        ["--format"]
    );
    assert_eq!(
        complete_str(&["/usr/bin/busybox", "status", "--form"]),
        ["--format"]
    );
}

#[test]
fn applet_invocation_matches_busybox_subcommand_invocation() {
    // Directly starting the `status` applet from any argv0 spelling must give
    // the same candidates as `busybox status`.
    for argv0 in [
        "status",
        "./status",
        "../bin/status",
        "/usr/local/bin/status",
        "status.exe",
        "/usr/bin/status.exe",
        "status.EXE",
        "./status.bat",
    ] {
        assert_eq!(
            complete_str(&[argv0, ""]),
            complete_str(&["/usr/bin/busybox", "status", ""]),
            "argv0 `{argv0}` changed the empty-word candidates"
        );
        assert_eq!(
            complete_str(&[argv0, "--format", ""]),
            complete_str(&["/usr/bin/busybox", "status", "--format", ""]),
            "argv0 `{argv0}` changed the space-form value candidates"
        );
        assert_eq!(
            complete_str(&[argv0, "--format="]),
            complete_str(&["/usr/bin/busybox", "status", "--format="]),
            "argv0 `{argv0}` changed the equals-form value candidates"
        );
    }
}

#[test]
fn direct_status_and_busybox_status_candidates_are_identical() {
    // Not just the values: subcommands, options, order, help, ids, tags and
    // hidden markers must agree between the two entry points, input by input.
    let pairs: &[(&[&str], &[&str])] = &[
        (&["busybox", "status", ""], &["status", ""]),
        (
            &["busybox", "status", "--format", ""],
            &["status", "--format", ""],
        ),
        (
            &["busybox", "status", "--format="],
            &["status", "--format="],
        ),
        (
            &["busybox", "status", "--format=t"],
            &["status", "--format=t"],
        ),
        (&["busybox", "status", "--"], &["status", "--"]),
        (&["busybox", "status", "--form"], &["status", "--form"]),
        (
            &["busybox", "status", "--format", "zzz"],
            &["status", "--format", "zzz"],
        ),
    ];
    for (busybox_path, direct_path) in pairs {
        assert_eq!(
            complete_full(direct_path),
            complete_full(busybox_path),
            "candidate metadata differs between {busybox_path:?} and {direct_path:?}"
        );
    }
}

#[test]
fn applet_does_not_leak_aggregate_or_other_applet_candidates() {
    // From within `status`, `busybox`'s `stop` subcommand and the applet roots
    // must never be offered.
    let candidates = complete_str(&["/usr/bin/status", ""]);
    assert!(!candidates.contains(&"stop".to_owned()));
    assert!(!candidates.contains(&"busybox".to_owned()));

    // And from `busybox`'s own scope, applet-internal options never appear.
    let candidates = complete_str(&["/usr/bin/busybox", ""]);
    assert!(!candidates.contains(&"--format".to_owned()));
    assert!(!candidates.contains(&"--force".to_owned()));
}

#[test]
fn unknown_applet_is_an_error_not_root_completion() {
    let mut cmd = multicall_cmd();
    let err = complete_raw(
        &mut cmd,
        &["/usr/bin/does-not-exist".into(), "".into()],
        1,
    )
    .expect_err("an unknown applet reports an error");
    assert!(
        err.to_string().contains("does-not-exist"),
        "the error names the unrecognized applet: {err}"
    );

    // Completion must not fall back to the aggregate root's subcommands; the
    // command is still usable afterwards.
    let candidates = complete_raw(&mut cmd, &["/usr/bin/busybox".into(), "".into()], 1).unwrap();
    assert_eq!(candidate_str(&candidates), ["status", "stop"]);
}

#[test]
fn applet_name_matching_is_case_sensitive() {
    let err = complete_str_result(&["/usr/bin/Status", ""]).expect_err("case must match exactly");
    assert!(err.to_string().contains("Status"));
    let err =
        complete_str_result(&["/usr/bin/STATUS.exe", ""]).expect_err("case must match exactly");
    assert!(err.to_string().contains("STATUS"));
}

fn complete_str_result(args: &[&str]) -> Result<Vec<String>, std::io::Error> {
    let args = args.iter().map(OsString::from).collect::<Vec<_>>();
    let index = args.len() - 1;
    clap_complete::engine::complete(&mut multicall_cmd(), args, index, None)
        .map(|candidates| candidate_str(&candidates))
}

#[test]
fn applet_aliases_are_respected() {
    // A visible alias declared on an applet selects it just like its canonical
    // name, matching how the parser resolves the argv0 file stem.
    let mut cmd = Command::new("busybox")
        .multicall(true)
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .subcommand(status_cmd().visible_alias("st"));
    let candidates = complete_raw(
        &mut cmd,
        &["/usr/bin/st".into(), "--format".into(), "".into()],
        2,
    )
    .unwrap();
    assert_eq!(candidate_str(&candidates), ["text", "json"]);
}

#[test]
fn value_terminator_works_through_applet() {
    // An option-level value terminator is handled inside the applet just like
    // in a non-multicall command: after it, ordinary completion resumes and the
    // terminator itself is never offered.
    let mut cmd = Command::new("busybox")
        .multicall(true)
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .subcommand(
            Command::new("status")
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["text", "json"])
                        .value_terminator(";"),
                )
                .arg(Arg::new("verbose").long("verbose").action(ArgAction::SetTrue)),
        );
    let args: Vec<OsString> = vec![
        "/usr/bin/status".into(),
        "--format".into(),
        ";".into(),
        "".into(),
    ];
    let candidates = complete_raw(&mut cmd, &args, 3).unwrap();
    let values = candidate_str(&candidates);
    assert!(values.contains(&"--format".to_owned()));
    assert!(values.contains(&"--verbose".to_owned()));
    assert!(!values.contains(&";".to_owned()));
    assert!(!values.contains(&"text".to_owned()));
}

#[test]
fn non_multicall_command_keeps_binary_name_behavior() {
    let mut cmd = Command::new("tool")
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .subcommand(Command::new("status"));
    // argv0 is consumed as the binary name; `stat` completes the subcommand.
    let candidates =
        complete_raw(&mut cmd, &["tool".into(), "stat".into()], 1).expect("completion succeeds");
    assert_eq!(candidate_str(&candidates), ["status"]);

    // A file-path argv0 unrelated to any subcommand is still just the binary
    // name and must not error as an unrecognized applet.
    let candidates =
        complete_raw(&mut cmd, &["/usr/bin/whatever".into(), "stat".into()], 1).unwrap();
    assert_eq!(candidate_str(&candidates), ["status"]);
}

#[test]
fn no_binary_name_behavior_is_unchanged() {
    let mut cmd = Command::new("exhaustive")
        .no_binary_name(true)
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .subcommand(Command::new("status"));
    // With `no_binary_name`, index 0 is the first logical word, not argv0.
    let candidates = complete_raw(&mut cmd, &["stat".into()], 0).unwrap();
    assert_eq!(candidate_str(&candidates), ["status"]);
}

#[test]
fn out_of_bounds_cursor_succeeds_empty() {
    // A cursor past every word names no word; this must not panic or error.
    let candidates = complete_str_at(&["/usr/bin/status", ""], 5);
    assert!(candidates.is_empty());
    let candidates = complete_str_at(&["/usr/bin/status", "--format", ""], 99);
    assert!(candidates.is_empty());
}

#[test]
fn terminator_and_words_after_it_do_not_error() {
    // The value terminator at the cursor resumes ordinary completion...
    assert!(complete_str(&["/usr/bin/status", "--"]).contains(&"--format".to_owned()));
    // ...and words following it never cause an unhandled error.
    assert!(complete_str(&["/usr/bin/status", "--", "anything"]).is_empty());
    assert!(complete_str(&["/usr/bin/status", "--", "anything", ""]).is_empty());
}

#[test]
fn failing_inputs_do_not_pollute_later_completions() {
    let mut cmd = multicall_cmd();

    // An unknown applet fails, then a valid applet completes normally.
    let _ = complete_raw(&mut cmd, &["/usr/bin/missing".into(), "".into()], 1);
    let candidates = complete_raw(&mut cmd, &["/usr/bin/status".into(), "".into()], 1).unwrap();
    assert_eq!(candidate_str(&candidates), ["--format"]);

    // Impossible value prefix, then valid completion again.
    let candidates =
        complete_raw(&mut cmd, &["/usr/bin/status".into(), "--format".into(), "zzz".into()], 2)
            .unwrap();
    assert!(candidates.is_empty());
    let candidates = complete_raw(&mut cmd, &["/usr/bin/status".into(), "--format".into(), "".into()], 2)
        .unwrap();
    assert_eq!(candidate_str(&candidates), ["text", "json"]);

    // Switching applets on the same Command never leaks candidates across them.
    let candidates = complete_raw(&mut cmd, &["/usr/bin/busybox".into(), "".into()], 1).unwrap();
    assert_eq!(candidate_str(&candidates), ["status", "stop"]);
}

#[test]
#[cfg(unix)]
fn non_utf8_argv0_does_not_error() {
    use std::os::unix::ffi::OsStringExt;

    // An argv0 whose file stem is not valid UTF-8 cannot name an applet; like
    // the parser it is treated as an ordinary binary name rather than failing,
    // so completion proceeds at the aggregate command's root.
    let mut cmd = multicall_cmd();
    let args = vec![
        OsString::from_vec(b"/usr/bin/\xff\xfe".to_vec()),
        OsString::from(""),
    ];
    let candidates = complete_raw(&mut cmd, &args, 1).expect("no unhandled error");
    assert_eq!(candidate_str(&candidates), ["busybox", "status"]);
}

#[test]
fn nested_stop_applet_is_scoped_to_busybox() {
    // `stop` only exists under `busybox`, so it appears there and nowhere else.
    assert_eq!(complete_str(&["/usr/bin/busybox", "sto"]), ["stop"]);
    assert_eq!(
        complete_str(&["/usr/bin/busybox", "stop", "--"]),
        ["--force"]
    );
    // There is no top-level `stop` applet link.
    complete_str_result(&["/usr/bin/stop", ""]).expect_err("`stop` is not a top-level applet");
}
