#![cfg(feature = "unstable-dynamic")]

use clap::{ArgGroup, Command};
use clap_complete::engine::CompletionCandidate;

/// Build the `tool` command used throughout these tests:
/// - automatic help and version flags are disabled
/// - the root declares a required, non-multiple `format` group whose members
///   are the value-free `--json`/`--yaml` flags (visible aliases `--js`/`--yml`)
/// - `--output` takes one of `file`/`stream`
/// - the `run` subcommand (visible alias `r`) declares its own required,
///   non-multiple group whose members are the value-free `--text`/`--binary`
///   flags (visible aliases `--txt`/`--bin`) and accepts the positional values
///   `job`/`log`
fn mutex_group_tool() -> Command {
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
        .group(
            ArgGroup::new("format")
                .args(["json", "yaml"])
                .required(true),
        )
        .arg(
            clap::Arg::new("output")
                .long("output")
                .value_parser(["file", "stream"]),
        )
        .subcommand(
            Command::new("run")
                .visible_alias("r")
                .disable_help_flag(true)
                .disable_version_flag(true)
                .disable_help_subcommand(true)
                .arg(
                    clap::Arg::new("text")
                        .long("text")
                        .visible_alias("txt")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::Arg::new("binary")
                        .long("binary")
                        .visible_alias("bin")
                        .action(clap::ArgAction::SetTrue),
                )
                .group(
                    ArgGroup::new("kind")
                        .args(["text", "binary"])
                        .required(true),
                )
                .arg(clap::Arg::new("item").value_parser(["job", "log"])),
        )
}

fn complete_group(cmd: &mut Command, args: &[&str]) -> Vec<CompletionCandidate> {
    let args: Vec<std::ffi::OsString> = args.iter().map(std::ffi::OsString::from).collect();
    let arg_index = args.len() - 1;
    clap_complete::engine::complete(cmd, args, arg_index, None).unwrap()
}

fn group_values(args: &[&str]) -> Vec<String> {
    let mut cmd = mutex_group_tool();
    complete_group(&mut cmd, args)
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn empty_word_lists_group_members_aliases_output_and_run() {
    assert_eq!(
        group_values(&["tool", ""]),
        ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"]
    );
}

#[test]
fn selecting_a_group_member_hides_the_whole_group() {
    // The selected member, the other member, and every alias disappear;
    // unrelated options and both subcommand entries stay available.
    let after_selection = ["run", "r", "--output"];
    for selected in ["--json", "--js", "--yaml", "--yml"] {
        assert_eq!(
            group_values(&["tool", selected, ""]),
            after_selection,
            "{selected} must close the whole group"
        );
    }
}

#[test]
fn repeating_a_selected_member_succeeds_without_restoring_group() {
    // Repeats of the same entry (main name or alias, in any order) succeed and
    // neither error nor bring group candidates back.
    for line in [
        &["tool", "--json", "--json", ""][..],
        &["tool", "--json", "--js", ""][..],
        &["tool", "--js", "--json", ""][..],
        &["tool", "--js", "--js", ""][..],
        &["tool", "--yaml", "--yml", ""][..],
        &["tool", "--yml", "--yaml", ""][..],
        // Completing the repeated flag itself is a successful empty result.
        &["tool", "--json", "--json"][..],
        &["tool", "--yaml", "--yml"][..],
    ] {
        let result = std::panic::catch_unwind(|| group_values(line));
        let values = result.unwrap_or_else(|_| panic!("completion failed for {line:?}"));
        for hidden in ["--json", "--js", "--yaml", "--yml"] {
            assert!(
                !values.contains(&hidden.to_owned()),
                "{hidden} must stay hidden after {line:?}, got {values:?}"
            );
        }
    }

    assert_eq!(
        group_values(&["tool", "--json", "--json", ""]),
        ["run", "r", "--output"]
    );
}

#[test]
fn output_value_completion() {
    assert_eq!(group_values(&["tool", "--output", ""]), ["file", "stream"]);
    assert_eq!(group_values(&["tool", "--output", "f"]), ["file"]);
    // `--output=` followed by another word leaves the option pending, so that
    // word is completed against its possible values.
    assert_eq!(group_values(&["tool", "--output=", "s"]), ["stream"]);
    // The attached form itself prefix-filters the possible values.
    assert_eq!(
        group_values(&["tool", "--output="]),
        ["--output=file", "--output=stream"]
    );
    assert_eq!(group_values(&["tool", "--output=s"]), ["--output=stream"]);

    // Prefixes without a match are successful empty results.
    assert!(group_values(&["tool", "--output", "x"]).is_empty());
    assert!(group_values(&["tool", "--output=x"]).is_empty());
}

#[test]
fn invalid_output_value_does_not_error_or_pollute() {
    // `pipe` is not a legal value, but the next (empty) word must complete
    // successfully and show the normal candidates.
    assert_eq!(
        group_values(&["tool", "--output=pipe", ""]),
        ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    assert_eq!(
        group_values(&["tool", "--output", "pipe", ""]),
        ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    // Completing the invalid word itself is a successful empty result.
    assert!(group_values(&["tool", "--output=pipe"]).is_empty());
    assert!(group_values(&["tool", "--output", "pipe"]).is_empty());
}

#[test]
fn run_level_lists_its_own_group_and_positionals() {
    assert_eq!(
        group_values(&["tool", "run", ""]),
        ["job", "log", "--text", "--txt", "--binary", "--bin"]
    );
    assert_eq!(group_values(&["tool", "run", "j"]), ["job"]);
    assert_eq!(group_values(&["tool", "run", "l"]), ["log"]);
}

#[test]
fn run_group_closes_after_selection_leaving_positionals() {
    for selected in ["--text", "--txt", "--binary", "--bin"] {
        assert_eq!(
            group_values(&["tool", "run", selected, ""]),
            ["job", "log"],
            "{selected} must close the run group"
        );
        // A positional prefix still completes after the group is chosen.
        assert_eq!(
            group_values(&["tool", "run", selected, "j"]),
            ["job"],
            "{selected} must not break positional completion"
        );
    }

    // Repeating a run-level member (main name or alias, in any order)
    // succeeds and keeps the group hidden.
    for line in [
        &["tool", "run", "--text", "--text", ""][..],
        &["tool", "run", "--text", "--txt", ""][..],
        &["tool", "run", "--bin", "--binary", ""][..],
    ] {
        assert_eq!(group_values(line), ["job", "log"], "{line:?}");
    }
    assert!(group_values(&["tool", "run", "--binary", "--binary"]).is_empty());
    assert!(group_values(&["tool", "run", "--txt", "--txt"]).is_empty());
}

#[test]
fn root_selection_does_not_leak_into_run() {
    // A root-level group choice, and root options generally, must not appear
    // once we descend into `run`.
    assert_eq!(
        group_values(&["tool", "--json", "run", ""]),
        ["job", "log", "--text", "--txt", "--binary", "--bin"]
    );
    assert_eq!(
        group_values(&["tool", "--js", "run", "--binary", ""]),
        ["job", "log"]
    );
    assert!(group_values(&["tool", "run", "--json"]).is_empty());
    for leaked in ["--json", "--js", "--yaml", "--yml", "--output", "run", "r"] {
        assert!(
            !group_values(&["tool", "run", ""]).contains(&leaked.to_owned()),
            "`{leaked}` must not leak into the run level"
        );
    }
}

#[test]
fn aliases_and_unmatched_prefixes_are_successful() {
    // A typed prefix is de-duplicated by argument, keeping the first spelling.
    // `--js` prefixes both `--json` and `--js`, so the canonical long name
    // wins; `--yml` prefixes only itself (`--yaml` does not start with
    // `--yml`), so the alias is offered. Every spelling is listed for the
    // empty word (see empty_word_lists_...).
    assert_eq!(group_values(&["tool", "--js"]), ["--json"]);
    assert_eq!(group_values(&["tool", "--yml"]), ["--yml"]);
    assert_eq!(group_values(&["tool", "--j"]), ["--json"]);
    assert_eq!(group_values(&["tool", "--y"]), ["--yaml"]);

    // Once the group is closed, alias prefixes match nothing successfully.
    assert!(group_values(&["tool", "--json", "--j"]).is_empty());
    assert!(group_values(&["tool", "--yaml", "--y"]).is_empty());

    // Unknown option prefixes at either level are successful empty results.
    assert!(group_values(&["tool", "--bogus"]).is_empty());
    assert!(group_values(&["tool", "run", "--bogus"]).is_empty());
    assert_eq!(group_values(&["tool", "run", "--t"]), ["--text"]);
    assert_eq!(group_values(&["tool", "run", "--b"]), ["--binary"]);
    // A prefix matching only the visible alias offers the alias; when both
    // spellings match, the canonical long wins de-duplication.
    assert_eq!(group_values(&["tool", "run", "--tx"]), ["--txt"]);
    assert_eq!(group_values(&["tool", "run", "--bi"]), ["--binary"]);
    // An unknown prior flag does not affect the next completion.
    assert_eq!(
        group_values(&["tool", "run", "--bogus", ""]),
        ["job", "log", "--text", "--txt", "--binary", "--bin"]
    );
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
fn filtering_preserves_retained_candidate_metadata() {
    // Group filtering may only shrink the candidate set; the retained
    // candidates keep their original value, help, id, tag, order, and
    // hidden state.
    let plain = candidate_metadata(complete_group(&mut mutex_group_tool(), &["tool", ""]));
    let filtered =
        candidate_metadata(complete_group(&mut mutex_group_tool(), &["tool", "--json", ""]));
    let expected: Vec<_> = plain
        .iter()
        .filter(|candidate| {
            candidate.0 == "run" || candidate.0 == "r" || candidate.0 == "--output"
        })
        .cloned()
        .collect();
    assert_eq!(filtered, expected);

    let plain = candidate_metadata(complete_group(&mut mutex_group_tool(), &["tool", "run", ""]));
    let filtered = candidate_metadata(complete_group(
        &mut mutex_group_tool(),
        &["tool", "run", "--binary", ""],
    ));
    let expected: Vec<_> = plain
        .iter()
        .filter(|candidate| candidate.0 == "job" || candidate.0 == "log")
        .cloned()
        .collect();
    assert_eq!(filtered, expected);
}

#[test]
fn failed_inputs_on_one_command_do_not_pollute_later_completion() {
    let mut cmd = mutex_group_tool();
    let complete = |cmd: &mut Command, args: &[&str]| {
        complete_group(cmd, args)
            .into_iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    };

    // Baseline.
    assert_eq!(
        complete(&mut cmd, &["tool", ""]),
        ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    // A group choice hides the group only for this call.
    assert_eq!(
        complete(&mut cmd, &["tool", "--json", ""]),
        ["run", "r", "--output"]
    );
    // An illegal option value, attached or separate, succeeds and leaves the
    // next completion untouched.
    assert_eq!(
        complete(&mut cmd, &["tool", "--output=pipe", ""]),
        ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    assert_eq!(
        complete(&mut cmd, &["tool", "--output", "bogus", ""]),
        ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    // Unknown options and no-match prefixes are empty successes.
    assert!(complete(&mut cmd, &["tool", "--bogus"]).is_empty());
    assert!(complete(&mut cmd, &["tool", "--output", "zzz"]).is_empty());
    // The group is fully offered again on a fresh, unfiltered line.
    assert_eq!(
        complete(&mut cmd, &["tool", ""]),
        ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    // Descending into the subcommand and coming back to the root are
    // independent: the run level shows no root state, then the root is whole.
    assert_eq!(
        complete(&mut cmd, &["tool", "--json", "run", ""]),
        ["job", "log", "--text", "--txt", "--binary", "--bin"]
    );
    assert_eq!(
        complete(&mut cmd, &["tool", ""]),
        ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"]
    );
}

#[test]
fn multiple_groups_keep_all_members_after_a_choice() {
    // A group that allows multiple members is not closed by choosing one of
    // them; the one-of filtering only applies to non-multiple groups.
    let mut cmd = Command::new("tool")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .arg(
            clap::Arg::new("json")
                .long("json")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("yaml")
                .long("yaml")
                .action(clap::ArgAction::SetTrue),
        )
        .group(
            ArgGroup::new("formats")
                .args(["json", "yaml"])
                .required(true)
                .multiple(true),
        );
    let values = complete_group(&mut cmd, &["tool", "--json", ""])
        .into_iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(values, ["--json", "--yaml"]);
}

#[test]
fn run_alias_path_equivalent_to_main_name_path() {
    // At every identical cursor position, entering `run` through its visible
    // alias `r` yields exactly the same candidate set, order, help, hidden
    // marker, grouping, and id as the canonical path.
    let equivalent: &[(&[&str], &[&str])] = &[
        (&["tool", "run", ""], &["tool", "r", ""]),
        (&["tool", "run", "j"], &["tool", "r", "j"]),
        (&["tool", "run", "l"], &["tool", "r", "l"]),
        (&["tool", "run", "--"], &["tool", "r", "--"]),
        (&["tool", "run", "--t"], &["tool", "r", "--t"]),
        (&["tool", "run", "--tx"], &["tool", "r", "--tx"]),
        (&["tool", "run", "--text", ""], &["tool", "r", "--text", ""]),
        (&["tool", "run", "--txt", ""], &["tool", "r", "--txt", ""]),
        (&["tool", "run", "--binary", "j"], &["tool", "r", "--binary", "j"]),
        (
            &["tool", "run", "--text", "--", ""],
            &["tool", "r", "--text", "--", ""],
        ),
    ];
    for (main, alias) in equivalent {
        let main_meta =
            candidate_metadata(complete_group(&mut mutex_group_tool(), main));
        let alias_meta =
            candidate_metadata(complete_group(&mut mutex_group_tool(), alias));
        assert_eq!(main_meta, alias_meta, "{main:?} vs {alias:?}");
    }

    // The empty-word order inside the subcommand: positionals first, then the
    // group's main names and visible aliases.
    let values = group_values(&["tool", "r", ""]);
    assert_eq!(values, ["job", "log", "--text", "--txt", "--binary", "--bin"]);

    // The alias entry at the root is the same command candidate in every
    // respect except its own spelling.
    let mut root = complete_group(&mut mutex_group_tool(), &["tool", ""]);
    root.retain(|candidate| {
        matches!(candidate.get_value().to_string_lossy().as_ref(), "run" | "r")
    });
    let root_meta = candidate_metadata(root);
    assert_eq!(root_meta.len(), 2);
    assert_eq!(root_meta[0].0, "run");
    assert_eq!(root_meta[1].0, "r");
    // Same tag, display order, and visibility.
    assert_eq!(root_meta[0].3, root_meta[1].3);
    assert_eq!(root_meta[0].4, root_meta[1].4);
    assert_eq!(root_meta[0].5, root_meta[1].5);
}

#[test]
fn root_selection_entering_through_alias_gives_isolated_subscope() {
    // A root format choice must not leak into the run level, whether run is
    // entered by its main name or its alias, and whether the format was given
    // by a main name or an alias.
    let sub_scope = ["job", "log", "--text", "--txt", "--binary", "--bin"];
    for line in [
        &["tool", "--json", "r", ""][..],
        &["tool", "--js", "r", ""][..],
        &["tool", "--yaml", "r", ""][..],
        &["tool", "--yml", "r", ""][..],
        &["tool", "--json", "run", ""][..],
        &["tool", "--js", "run", ""][..],
    ] {
        assert_eq!(group_values(line), sub_scope, "{line:?}");
        assert_eq!(
            candidate_metadata(complete_group(&mut mutex_group_tool(), line)),
            candidate_metadata(complete_group(
                &mut mutex_group_tool(),
                &["tool", "run", ""]
            )),
            "{line:?} must be identical to a plain `tool run` entry"
        );
    }

    // A root `--output` choice in any form likewise does not leak, and after a
    // group selection at the sub level only positionals remain.
    for line in [
        &["tool", "--output", "file", "r", ""][..],
        &["tool", "--output=file", "r", ""][..],
        &["tool", "--output", "stream", "run", ""][..],
        &["tool", "--output=stream", "run", ""][..],
    ] {
        assert_eq!(group_values(line), sub_scope, "{line:?}");
    }
    for line in [
        &["tool", "--json", "r", "--text", ""][..],
        &["tool", "--js", "r", "--txt", ""][..],
        &["tool", "--yaml", "run", "--binary", ""][..],
        &["tool", "--output", "file", "r", "--bin", ""][..],
    ] {
        assert_eq!(group_values(line), ["job", "log"], "{line:?}");
    }
}

#[test]
fn output_value_forms_do_not_pollute_later_completion() {
    // Space-separated, equals-separated, and illegal `--output` values must
    // not pollute a later descent through the main name or the alias, nor the
    // next root completion.
    let root_scope = ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"];
    let sub_scope = ["job", "log", "--text", "--txt", "--binary", "--bin"];
    assert_eq!(
        group_values(&["tool", "--output", "file", "run", ""]),
        sub_scope
    );
    assert_eq!(
        group_values(&["tool", "--output=file", "r", ""]),
        sub_scope
    );
    assert_eq!(
        group_values(&["tool", "--output", "pipe", "run", ""]),
        sub_scope
    );
    assert_eq!(
        group_values(&["tool", "--output=pipe", "r", ""]),
        sub_scope
    );
    assert_eq!(
        group_values(&["tool", "--output", "pipe", ""]),
        root_scope
    );
    assert_eq!(
        group_values(&["tool", "--output=pipe", ""]),
        root_scope
    );
    // Illegal value *prefixes*, spaced or attached, also must not pollute a
    // subsequent descent through either entry spelling.
    assert_eq!(
        group_values(&["tool", "--output", "p", "r", ""]),
        sub_scope
    );
    assert_eq!(
        group_values(&["tool", "--output=p", "run", ""]),
        sub_scope
    );
    // Completing the illegal value prefix itself is a successful empty
    // result, in both the spaced and attached forms.
    assert!(group_values(&["tool", "--output", "p"]).is_empty());
    assert!(group_values(&["tool", "--output=p"]).is_empty());
}

#[test]
fn unmatched_and_illegal_alias_prefixes_are_empty_successes() {
    // No-match prefixes of subcommand names or positional values succeed with
    // no candidates at either level, through either entry spelling.
    assert!(group_values(&["tool", "rx"]).is_empty());
    assert!(group_values(&["tool", "rr"]).is_empty());
    assert!(group_values(&["tool", "run", "x"]).is_empty());
    assert!(group_values(&["tool", "r", "x"]).is_empty());

    // Illegal long-option alias prefixes (matching neither a main name nor a
    // visible alias) succeed empty at both levels.
    assert!(group_values(&["tool", "--jx"]).is_empty());
    assert!(group_values(&["tool", "--ymx"]).is_empty());
    assert!(group_values(&["tool", "--jsonx"]).is_empty());
    assert!(group_values(&["tool", "run", "--txtx"]).is_empty());
    assert!(group_values(&["tool", "r", "--binn"]).is_empty());

    // Once the root group is closed, even valid prefixes of its members match
    // nothing successfully.
    assert!(group_values(&["tool", "--json", "--js"]).is_empty());
    assert!(group_values(&["tool", "--yaml", "--y"]).is_empty());
    // Same for the run-level group.
    assert!(group_values(&["tool", "r", "--text", "--t"]).is_empty());
    assert!(group_values(&["tool", "run", "--bin", "--b"]).is_empty());
}

#[test]
fn repeating_a_selected_member_always_succeeds() {
    // Repeats of a selected member, main name or alias in any order, never
    // error and never restore the group, at the root and inside the run level
    // (entered through either entry).
    for line in [
        &["tool", "--json", "--js", ""][..],
        &["tool", "--js", "--json", ""][..],
        &["tool", "--yml", "--yaml", ""][..],
        &["tool", "r", "--text", "--txt", ""][..],
        &["tool", "run", "--txt", "--text", ""][..],
        &["tool", "r", "--binary", "--bin", ""][..],
        &["tool", "r", "--bin", "--binary", ""][..],
    ] {
        let result = std::panic::catch_unwind(|| group_values(line));
        let values = result.unwrap_or_else(|_| panic!("completion failed for {line:?}"));
        let at_run_level = line[1] == "r" || line[1] == "run";
        if at_run_level {
            assert_eq!(values, ["job", "log"], "{line:?}");
        } else {
            assert_eq!(values, ["run", "r", "--output"], "{line:?}");
        }
    }

    // Completing the repeated word itself is a successful empty result.
    assert!(group_values(&["tool", "--json", "--js"]).is_empty());
    assert!(group_values(&["tool", "r", "--text", "--txt"]).is_empty());
}

#[test]
fn run_terminator_offers_only_positionals() {
    // After the terminator only job/log are offered, identically through the
    // main name and the alias, including full metadata.
    for line in [
        &["tool", "run", "--", ""][..],
        &["tool", "r", "--", ""][..],
        &["tool", "run", "--text", "--", ""][..],
        &["tool", "r", "--bin", "--", ""][..],
    ] {
        assert_eq!(group_values(line), ["job", "log"], "{line:?}");
    }
    assert_eq!(
        candidate_metadata(complete_group(&mut mutex_group_tool(), &["tool", "run", "--", ""])),
        candidate_metadata(complete_group(&mut mutex_group_tool(), &["tool", "r", "--", ""])),
    );

    // Prefix filtering still applies, and a consumed positional slot yields a
    // successful empty result rather than erroring or switching levels.
    assert_eq!(group_values(&["tool", "r", "--", "j"]), ["job"]);
    assert_eq!(group_values(&["tool", "run", "--", "l"]), ["log"]);
    assert!(group_values(&["tool", "run", "--", "job", ""]).is_empty());

    // After the boundary, subcommand aliases, option-style words, unknown
    // words, and another terminator are positional values matching nothing:
    // successful empty results with no fallback to the root level.
    for word in [
        "r", "run", "deploy", "--text", "--txt", "--binary", "--bin", "--json", "--output",
        "--bogus", "-x", "--",
    ] {
        assert!(
            group_values(&["tool", "r", "--", word]).is_empty(),
            "`{word}` after the run-level terminator must match nothing"
        );
        assert!(
            group_values(&["tool", "run", "--", word]).is_empty(),
            "`{word}` after the run-level terminator must match nothing"
        );
    }
}

#[test]
fn successive_completions_on_one_command_are_isolated() {
    // Drive one command instance through a sequence of unrelated completion
    // requests; each must be computed from its own arguments alone.
    let mut cmd = mutex_group_tool();
    let complete = |cmd: &mut Command, args: &[&str]| {
        complete_group(cmd, args)
            .into_iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
    };

    let root = ["run", "r", "--json", "--js", "--yaml", "--yml", "--output"];
    let sub = ["job", "log", "--text", "--txt", "--binary", "--bin"];

    assert_eq!(complete(&mut cmd, &["tool", ""]), root);
    // A root group choice only affects this request.
    assert_eq!(complete(&mut cmd, &["tool", "--json", ""]), ["run", "r", "--output"]);
    // An illegal output value followed by an alias descent stays isolated.
    assert_eq!(complete(&mut cmd, &["tool", "--output=pipe", "r", ""]), sub);
    assert_eq!(complete(&mut cmd, &["tool", "--output", "bogus", "run", ""]), sub);
    // A no-match prefix is an empty success.
    assert!(complete(&mut cmd, &["tool", "--bogus"]).is_empty());
    assert!(complete(&mut cmd, &["tool", "r", "--nope"]).is_empty());
    // A terminated sub level offers its positionals.
    assert_eq!(complete(&mut cmd, &["tool", "r", "--", ""]), ["job", "log"]);
    // Boundary words never fall back to the root.
    assert!(complete(&mut cmd, &["tool", "r", "--", "run"]).is_empty());
    assert!(complete(&mut cmd, &["tool", "r", "--", "--json"]).is_empty());
    // The root is offered whole again on a fresh line, and so is the run
    // level with no root state leaking.
    assert_eq!(complete(&mut cmd, &["tool", ""]), root);
    assert_eq!(complete(&mut cmd, &["tool", "r", ""]), sub);
    assert_eq!(complete(&mut cmd, &["tool", "run", ""]), sub);
}
