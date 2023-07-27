use std::os::unix::prelude::CommandExt;
use std::process::Command;

use assert_cmd::prelude::*;
use assert_fs::TempDir;

/// If just executing the `plow` command without any arguments, we should exit with code `0`.
#[test]
fn no_arguments_correct_exit_code() {
    let out = Command::cargo_bin("plow")
        .unwrap()
        .assert();

    out.code(0);
}

/// Test that `plow init --field @test/testname` generates the correct file in empty directory.
// TODO: In the future this should be moved to the command `plow new` and the command should fail in an empty directory.
#[test]
fn plow_init_field_empty_dir() {
    let tmp_dir = TempDir::new().unwrap();

    let out = Command::cargo_bin("plow").unwrap()
        .arg("init")
        .arg("--field")
        .arg("@test/testname")
        .current_dir(tmp_dir.path())
        .unwrap()
        .assert();

    out.code(0);

    let outfile_path = tmp_dir.path().join("testname.ttl");
    assert!(outfile_path.exists());

    let generated_ontology_contents = std::fs::read_to_string(outfile_path).unwrap();
    insta::assert_snapshot!(generated_ontology_contents);
}