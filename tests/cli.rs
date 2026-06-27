use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_charon6"))
        .args(args)
        .output()
        .expect("failed to run charon6")
}

#[test]
fn help_exits_zero_and_prints_usage() {
    let output = run(&["--help"]);

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: charon6"));
    assert!(stdout.contains("--cidr"));
}

#[test]
fn invalid_cidr_exits_with_usage_error() {
    let output = run(&["--cidr", "not-a-cidr"]);

    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value 'not-a-cidr'"));
}

#[test]
fn missing_cidr_value_exits_with_usage_error() {
    let output = run(&["--cidr"]);

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn unknown_flag_exits_with_usage_error() {
    let output = run(&["--nope"]);

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn missing_required_cidr_exits_with_usage_error() {
    let output = run(&[]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--cidr"));
}
