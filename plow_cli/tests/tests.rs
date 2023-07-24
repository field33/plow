use std::os::unix::prelude::CommandExt;
use std::process::Command;

use assert_cmd::prelude::*;

/// If just executing the `plow` command without any arguments, we should exit with code `0`.
#[test]
fn no_arguments_correct_exit_code() {
    let out = Command::cargo_bin("plow")
        .unwrap()
        .assert();

    out.code(0);
}