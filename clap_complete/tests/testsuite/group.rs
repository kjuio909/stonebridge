#![cfg(feature = "unstable-dynamic")]

use clap::{ArgGroup, Command};
use clap_complete::engine::CompletionCandidate;

/// Build the `tool` command used throughout these tests:
/// - automatic help and version flags are disabled
/// - the root declares a required, non-multiple `format` group whose members
///   are the value-free `--json`/`--yaml` flags (visible aliases `--js`/`--yml`)
/// - `--output` takes one of `file`/`stream`
/// - the `run` subcommand declares its own required, non-multiple group
///   (`--text`/`--binary`) and accepts the positional values `job`/`log`
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
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );
}

#[test]
fn selecting_a_group_member_hides_the_whole_group() {
    // The selected member, the other member, and every alias disappear;
    // unrelated options and the subcommand stay available.
    let after_selection = ["run", "--output"];
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
        ["run", "--output"]
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
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    assert_eq!(
        group_values(&["tool", "--output", "pipe", ""]),
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    // Completing the invalid word itself is a successful empty result.
    assert!(group_values(&["tool", "--output=pipe"]).is_empty());
    assert!(group_values(&["tool", "--output", "pipe"]).is_empty());
}

#[test]
fn run_level_lists_its_own_group_and_positionals() {
    assert_eq!(
        group_values(&["tool", "run", ""]),
        ["job", "log", "--text", "--binary"]
    );
    assert_eq!(group_values(&["tool", "run", "j"]), ["job"]);
    assert_eq!(group_values(&["tool", "run", "l"]), ["log"]);
}

#[test]
fn run_group_closes_after_selection_leaving_positionals() {
    for selected in ["--text", "--binary"] {
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

    // Repeating a run-level member succeeds and keeps the group hidden.
    assert_eq!(
        group_values(&["tool", "run", "--text", "--text", ""]),
        ["job", "log"]
    );
    assert!(group_values(&["tool", "run", "--binary", "--binary"]).is_empty());
}

#[test]
fn root_selection_does_not_leak_into_run() {
    // A root-level group choice, and root options generally, must not appear
    // once we descend into `run`.
    assert_eq!(
        group_values(&["tool", "--json", "run", ""]),
        ["job", "log", "--text", "--binary"]
    );
    assert_eq!(
        group_values(&["tool", "--js", "run", "--binary", ""]),
        ["job", "log"]
    );
    assert!(group_values(&["tool", "run", "--json"]).is_empty());
    for leaked in ["--json", "--js", "--yaml", "--yml", "--output", "run"] {
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
    // An unknown prior flag does not affect the next completion.
    assert_eq!(
        group_values(&["tool", "run", "--bogus", ""]),
        ["job", "log", "--text", "--binary"]
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
        .filter(|candidate| candidate.0 == "run" || candidate.0 == "--output")
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
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    // A group choice hides the group only for this call.
    assert_eq!(complete(&mut cmd, &["tool", "--json", ""]), ["run", "--output"]);
    // An illegal option value, attached or separate, succeeds and leaves the
    // next completion untouched.
    assert_eq!(
        complete(&mut cmd, &["tool", "--output=pipe", ""]),
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    assert_eq!(
        complete(&mut cmd, &["tool", "--output", "bogus", ""]),
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    // Unknown options and no-match prefixes are empty successes.
    assert!(complete(&mut cmd, &["tool", "--bogus"]).is_empty());
    assert!(complete(&mut cmd, &["tool", "--output", "zzz"]).is_empty());
    // The group is fully offered again on a fresh, unfiltered line.
    assert_eq!(
        complete(&mut cmd, &["tool", ""]),
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
    );
    // Descending into the subcommand and coming back to the root are
    // independent: the run level shows no root state, then the root is whole.
    assert_eq!(
        complete(&mut cmd, &["tool", "--json", "run", ""]),
        ["job", "log", "--text", "--binary"]
    );
    assert_eq!(
        complete(&mut cmd, &["tool", ""]),
        ["run", "--json", "--js", "--yaml", "--yml", "--output"]
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
