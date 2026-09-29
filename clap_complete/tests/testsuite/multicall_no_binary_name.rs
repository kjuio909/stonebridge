#![cfg(feature = "unstable-dynamic")]

//! Dynamic completion for a multicall binary that also sets
//! [`Command::no_binary_name`].
//!
//! In that mode the executable name (`argv[0]`) must still resolve the applet,
//! while cursor semantics stay those of a no-binary-name command: logical words
//! begin after the resolved applet and the consumed `argv[0]` is never a
//! completion target. Completion through a nested applet (`busybox status`) and
//! through the matching top-level applet (`status`) has to be identical,
//! regardless of how `argv[0]` was spelled.

use std::ffi::OsString;
use std::io;

use clap::{builder::PossibleValue, Arg, ArgAction, ArgGroup, Command};
use clap_complete::engine::CompletionCandidate;

/// The inheritable `--config <dev|prod>` argument.
///
/// It is global so applets inherit it, and every command that hosts it puts the
/// argument in a single-member, non-multiple group: that gives the parser-level
/// "set at most once" semantics completion already understands, so a completed
/// `--config` is not offered again at the same level.
fn config_arg() -> Arg {
    Arg::new("config")
        .long("config")
        .global(true)
        .value_parser([
            PossibleValue::new("dev").help("Use the dev configuration"),
            PossibleValue::new("prod").help("Use the production configuration"),
        ])
}

/// The `status` applet, declared both under the `busybox` launcher and as a
/// top-level applet so both entry points are comparable.
fn status_cmd() -> Command {
    Command::new("status")
        .about("Show service status")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .group(ArgGroup::new("config-group").arg("config"))
        .arg(
            Arg::new("format").long("format").value_parser([
                PossibleValue::new("text").help("Plain text output"),
                PossibleValue::new("json").help("JSON output"),
            ]),
        )
        .arg(config_arg())
        .visible_alias("st")
        .alias("legacy-status")
}

/// `stop` only exists as a nested applet of `busybox`.
fn stop_cmd() -> Command {
    Command::new("stop")
        .about("Stop the service")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .group(ArgGroup::new("config-group").arg("config"))
        .arg(
            Arg::new("force")
                .long("force")
                .action(ArgAction::SetTrue)
                .help("Stop even while busy"),
        )
        .arg(config_arg())
}

/// The launcher applet reached when invoked as `busybox` or its visible alias
/// `bb`; it hosts the nested `status` and `stop` applets.
fn busybox_applet() -> Command {
    Command::new("busybox")
        .about("Multi-call launcher")
        .visible_alias("bb")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .group(ArgGroup::new("config-group").arg("config"))
        .arg(config_arg())
        .subcommand(status_cmd())
        .subcommand(stop_cmd())
}

/// The aggregate multicall command, optionally also setting `no_binary_name`.
/// Every assertion in this file runs under both settings and must agree.
fn aggregate(no_binary_name: bool) -> Command {
    Command::new("busybox")
        .multicall(true)
        .no_binary_name(no_binary_name)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .subcommand(busybox_applet())
        // The `status` applet is also reachable through a link named after it.
        .subcommand(status_cmd())
}

fn os_args(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

/// Complete the last word on a freshly built aggregate command.
fn complete(no_binary_name: bool, args: &[&str]) -> io::Result<Vec<CompletionCandidate>> {
    complete_at(no_binary_name, args, args.len() - 1)
}

/// Complete the word at `index` on a freshly built aggregate command.
fn complete_at(
    no_binary_name: bool,
    args: &[&str],
    index: usize,
) -> io::Result<Vec<CompletionCandidate>> {
    clap_complete::engine::complete(&mut aggregate(no_binary_name), os_args(args), index, None)
}

fn values(no_binary_name: bool, args: &[&str]) -> Vec<String> {
    let values = complete(no_binary_name, args).expect("completion succeeded");
    candidate_str(&values)
}

fn candidate_str(candidates: &[CompletionCandidate]) -> Vec<String> {
    candidates
        .iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

/// Both values of the setting under test.
const BOTH: [bool; 2] = [false, true];

#[test]
fn busybox_empty_word_lists_only_its_applets_and_config() {
    for no_binary_name in BOTH {
        assert_eq!(
            values(no_binary_name, &["/usr/bin/busybox", ""]),
            ["status", "st", "stop", "--config"],
            "no_binary_name={no_binary_name}"
        );
        // Help and version are disabled, and the root never sees inner flags.
        let shown = values(no_binary_name, &["/usr/bin/busybox", ""]);
        for leaked in ["--help", "--version", "-V", "--format", "--force", "legacy-status"] {
            assert!(
                !shown.contains(&leaked.to_owned()),
                "{leaked} leaked at busybox root (no_binary_name={no_binary_name})"
            );
        }
    }
}

#[test]
fn argv0_spellings_selecting_busybox_agree_at_every_level() {
    // File name, a path with directories, a `.exe` suffix, and the visible
    // alias must all resolve to the busybox applet and produce identical
    // candidates, word for word, at the root and on deeper levels.
    let argv0s = [
        "busybox",
        "./busybox",
        "../bin/busybox",
        "/usr/local/bin/busybox",
        "busybox.exe",
        "/usr/bin/busybox.exe",
        "bb",
        "/usr/bin/bb",
    ];
    // `(suffix words after argv0)` addressed at its last word.
    let levels: &[&[&str]] = &[
        &[""],
        &["s"],
        &["st"],
        &["sto"],
        &["status", ""],
        &["st", ""],
        &["status", "--"],
        &["status", "--format", ""],
        &["status", "--format", "j"],
        &["status", "--format="],
        &["status", "--format=t"],
        &["status", "--config", ""],
        &["status", "--config", "p"],
        &["status", "--config="],
        &["status", "--config=dev", ""],
        &["stop", ""],
        &["stop", "--"],
    ];

    for no_binary_name in BOTH {
        for level in levels {
            let mut baseline_args = vec!["/usr/bin/busybox"];
            baseline_args.extend_from_slice(level);
            let baseline = complete(no_binary_name, &baseline_args).expect("baseline completed");
            for argv0 in argv0s {
                let mut args = vec![argv0];
                args.extend_from_slice(level);
                let candidates = complete(no_binary_name, &args).expect("completion succeeded");
                assert_eq!(
                    candidate_str(&candidates),
                    candidate_str(&baseline),
                    "argv0 `{argv0}` with suffix {level:?} differs (no_binary_name={no_binary_name})"
                );
                assert_eq!(
                    candidates, baseline,
                    "metadata differs for argv0 `{argv0}` suffix {level:?} (no_binary_name={no_binary_name})"
                );
            }
        }
    }
}

#[test]
fn argv0_spellings_selecting_status_agree_with_nested_busybox_status() {
    let status_argv0s = [
        "status",
        "./status",
        "/usr/bin/status",
        "status.exe",
        "/usr/bin/status.exe",
        "st",
        "/usr/bin/st",
        "legacy-status",
        "/usr/bin/legacy-status",
    ];
    let levels: &[&[&str]] = &[
        &[""],
        &["--"],
        &["--form"],
        &["--format", ""],
        &["--format", "j"],
        &["--format="],
        &["--format=json"],
        &["--config", ""],
        &["--config", "d"],
        &["--config="],
        &["--config=dev", ""],
        &["--", "anything"],
    ];

    for no_binary_name in BOTH {
        for level in levels {
            let nested = {
                let mut args = vec!["/usr/bin/busybox", "status"];
                args.extend_from_slice(level);
                complete(no_binary_name, &args).expect("nested completed")
            };
            for argv0 in status_argv0s {
                let mut args = vec![argv0];
                args.extend_from_slice(level);
                let direct = complete(no_binary_name, &args).expect("direct completed");
                assert_eq!(
                    direct, nested,
                    "direct argv0 `{argv0}` suffix {level:?} != `busybox status` (no_binary_name={no_binary_name})"
                );
            }
        }
    }
}

#[test]
fn entering_status_scopes_candidates_to_that_applet() {
    for no_binary_name in BOTH {
        for entry in &["status", "st"] {
            assert_eq!(
                values(no_binary_name, &["/usr/bin/busybox", entry, ""]),
                ["--format", "--config"],
                "entry `{entry}` (no_binary_name={no_binary_name})"
            );
            assert_eq!(
                values(no_binary_name, &["/usr/bin/busybox", entry, "--"]),
                ["--format", "--config"],
                "entry `{entry}` after -- (no_binary_name={no_binary_name})"
            );
            // The sibling applet and the launcher root never leak in.
            let shown = values(no_binary_name, &["/usr/bin/busybox", entry, ""]);
            assert!(!shown.contains(&"stop".to_owned()));
            assert!(!shown.contains(&"busybox".to_owned()));
            assert!(!shown.contains(&"legacy-status".to_owned()));
        }
        assert_eq!(
            values(no_binary_name, &["/usr/bin/busybox", "status", "--format", ""]),
            ["text", "json"]
        );
        assert_eq!(
            values(no_binary_name, &["/usr/bin/busybox", "status", "--format", "t"]),
            ["text"]
        );
        assert_eq!(
            values(no_binary_name, &["/usr/bin/busybox", "status", "--config", ""]),
            ["dev", "prod"]
        );
        assert_eq!(
            values(no_binary_name, &["/usr/bin/busybox", "status", "--config="]),
            ["--config=dev", "--config=prod"]
        );
    }
}

#[test]
fn hidden_alias_enters_but_is_never_suggested() {
    for no_binary_name in BOTH {
        // Prefixes of the hidden alias, including its full name while still the
        // word under the cursor, suggest nothing.
        for prefix in ["", "l", "leg", "legacy", "legacy-status"] {
            let shown = values(no_binary_name, &["/usr/bin/busybox", prefix]);
            assert!(
                !shown.contains(&"legacy-status".to_owned()),
                "hidden alias suggested for prefix `{prefix}` (no_binary_name={no_binary_name})"
            );
        }

        // As a committed word it descends exactly like `status`, including via
        // an argv0 link named after the alias.
        assert_eq!(
            values(no_binary_name, &["/usr/bin/busybox", "legacy-status", ""]),
            ["--format", "--config"]
        );
        assert_eq!(
            values(
                no_binary_name,
                &["/usr/bin/busybox", "legacy-status", "--format", ""]
            ),
            ["text", "json"]
        );
        assert_eq!(
            values(no_binary_name, &["/usr/bin/legacy-status", ""]),
            values(no_binary_name, &["/usr/bin/status", ""]),
        );
    }
}

#[test]
fn config_keeps_prefix_in_space_and_equals_forms_and_is_not_reused() {
    for no_binary_name in BOTH {
        // Typed prefixes are retained in both forms.
        assert_eq!(
            values(no_binary_name, &["/usr/bin/status", "--config", "d"]),
            ["dev"]
        );
        assert_eq!(
            values(no_binary_name, &["/usr/bin/status", "--config", "p"]),
            ["prod"]
        );
        assert_eq!(
            values(no_binary_name, &["/usr/bin/status", "--config=d"]),
            ["--config=dev"]
        );
        assert_eq!(
            values(no_binary_name, &["/usr/bin/status", "--config=z"]),
            Vec::<String>::new()
        );

        // After a completed value, neither the flag nor its values come back,
        // in either form, at the status level.
        for after in [
            vec!["/usr/bin/status", "--config", "dev", ""],
            vec!["/usr/bin/status", "--config=dev", ""],
            vec!["/usr/bin/busybox", "status", "--config", "dev", ""],
            vec!["/usr/bin/busybox", "status", "--config=dev", ""],
        ] {
            let shown = values(no_binary_name, &after);
            assert!(
                !shown.contains(&"--config".to_owned())
                    && !shown.contains(&"dev".to_owned())
                    && !shown.contains(&"prod".to_owned()),
                "config re-suggested after completion: {after:?} (no_binary_name={no_binary_name})"
            );
            assert!(shown.contains(&"--format".to_owned()));
        }

        // The same at the busybox launcher level.
        assert_eq!(
            values(no_binary_name, &["/usr/bin/busybox", "--config", "dev", ""]),
            ["status", "st", "stop"]
        );
    }
}

#[test]
fn nested_and_direct_status_paths_have_identical_metadata() {
    // Value, order, help, hidden flag, tag and id must all agree between the
    // two comparable paths, input by input.
    let pairs: &[(&[&str], &[&str])] = &[
        (&["busybox", "status", ""], &["status", ""]),
        (&["busybox", "st", ""], &["st", ""]),
        (&["busybox", "legacy-status", ""], &["legacy-status", ""]),
        (&["busybox", "status", "--"], &["status", "--"]),
        (&["busybox", "status", "--form"], &["status", "--form"]),
        (
            &["busybox", "status", "--format", ""],
            &["status", "--format", ""],
        ),
        (&["busybox", "status", "--format="], &["status", "--format="]),
        (&["busybox", "status", "--format=t"], &["status", "--format=t"]),
        (
            &["busybox", "status", "--config", ""],
            &["status", "--config", ""],
        ),
        (&["busybox", "status", "--config="], &["status", "--config="]),
        (&["busybox", "status", "--config=d"], &["status", "--config=d"]),
        (
            &["busybox", "status", "--config", "zzz"],
            &["status", "--config", "zzz"],
        ),
    ];
    for no_binary_name in BOTH {
        for (nested, direct) in pairs {
            assert_eq!(
                complete(no_binary_name, direct).unwrap(),
                complete(no_binary_name, nested).unwrap(),
                "metadata differs for {nested:?} vs {direct:?} (no_binary_name={no_binary_name})"
            );
        }

        // Entering through the hidden alias yields the same inner candidates as
        // through the canonical name.
        assert_eq!(
            complete(no_binary_name, &["busybox", "legacy-status", ""]).unwrap(),
            complete(no_binary_name, &["busybox", "status", ""]).unwrap(),
        );
    }
}

#[test]
fn unknown_or_wrong_case_applet_is_an_error_not_root_completion() {
    for no_binary_name in BOTH {
        for argv0 in ["/usr/bin/does-not-exist", "/x/Status", "/x/STATUS.exe", "/x/stop"] {
            // `stop` is only a nested applet of busybox, not a top-level applet.
            let err = complete(no_binary_name, &[argv0, ""]).expect_err("unrecognized applet");
            let stem = std::path::Path::new(argv0)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            assert!(
                err.to_string().contains(&stem),
                "error `{err}` should name `{stem}`"
            );
        }
    }
}

#[test]
fn out_of_bounds_terminator_bad_value_and_prefix_return_empty() {
    for no_binary_name in BOTH {
        // Cursor past the last word.
        assert!(complete_at(no_binary_name, &["/usr/bin/status", ""], 9).unwrap().is_empty());
        assert!(
            complete_at(no_binary_name, &["/usr/bin/status", "--format", ""], 7)
                .unwrap()
                .is_empty()
        );
        // Cursor on the consumed argv[0].
        assert!(complete_at(no_binary_name, &["/usr/bin/status", ""], 0).unwrap().is_empty());
        assert!(complete_at(no_binary_name, &["/usr/bin/busybox", ""], 0).unwrap().is_empty());

        // Words after the value terminator are escaped positionals.
        assert!(values(no_binary_name, &["/usr/bin/status", "--", "anything"]).is_empty());
        assert!(
            values(no_binary_name, &["/usr/bin/status", "--", "anything", ""]).is_empty()
        );

        // Illegal values in either form.
        assert!(values(no_binary_name, &["/usr/bin/status", "--format", "zzz"]).is_empty());
        assert!(values(no_binary_name, &["/usr/bin/status", "--format=zzz"]).is_empty());
        assert!(values(no_binary_name, &["/usr/bin/status", "--config", "zzz"]).is_empty());

        // Prefix with no match, and an unknown nested word at the launcher.
        assert!(values(no_binary_name, &["/usr/bin/status", "--formx"]).is_empty());
        assert!(values(no_binary_name, &["/usr/bin/busybox", "xyz"]).is_empty());
    }
}

#[test]
fn earlier_failed_or_other_applet_calls_do_not_leak_into_reused_command() {
    for no_binary_name in BOTH {
        let mut cmd = aggregate(no_binary_name);

        // An unrecognized applet errors first.
        let err = clap_complete::engine::complete(
            &mut cmd,
            os_args(&["/usr/bin/missing", ""]),
            1,
            None,
        )
        .expect_err("unknown applet errors");
        assert!(err.to_string().contains("missing"));

        // Then descend through the hidden alias into status.
        let via_hidden = clap_complete::engine::complete(
            &mut cmd,
            os_args(&["/usr/bin/busybox", "legacy-status", ""]),
            2,
            None,
        )
        .unwrap();
        assert_eq!(candidate_str(&via_hidden), ["--format", "--config"]);

        // Then switch to the other applet (the launcher root).
        let at_root = clap_complete::engine::complete(
            &mut cmd,
            os_args(&["/usr/bin/busybox", ""]),
            1,
            None,
        )
        .unwrap();
        assert_eq!(candidate_str(&at_root), ["status", "st", "stop", "--config"]);

        // A bad config value on the status applet.
        let bad = clap_complete::engine::complete(
            &mut cmd,
            os_args(&["/usr/bin/status", "--config", "zzz"]),
            2,
            None,
        )
        .unwrap();
        assert!(bad.is_empty());

        // Finally, a legal path on the same object must be byte-for-byte the
        // same as on a freshly built command, with no cross-applet leakage.
        let reused = clap_complete::engine::complete(
            &mut cmd,
            os_args(&["/usr/bin/status", "--config", ""]),
            2,
            None,
        )
        .unwrap();
        let fresh = complete(no_binary_name, &["/usr/bin/status", "--config", ""]).unwrap();
        assert_eq!(reused, fresh);

        let reused_root = clap_complete::engine::complete(
            &mut cmd,
            os_args(&["/usr/bin/busybox", "status", ""]),
            2,
            None,
        )
        .unwrap();
        assert_eq!(
            candidate_str(&reused_root),
            ["--format", "--config"],
            "`stop` or other applet state leaked into a reused command"
        );
    }
}

#[test]
fn setting_both_flags_does_not_panic() {
    // The combination used to trip a debug assertion in clap; building and
    // completing must work in debug builds.
    let candidates = complete(true, &["/usr/bin/busybox", ""]).unwrap();
    assert_eq!(candidate_str(&candidates), ["status", "st", "stop", "--config"]);
}
