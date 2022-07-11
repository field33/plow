use std::path::PathBuf;

/// Constructs path based on whether we are running via `bazel test` or `cargo test`
pub fn tests_filepath(filename: &str) -> PathBuf {
    for var in std::env::vars() {
        // Running in bazel test
        if var.0 == "RUN_UNDER_RUNFILES" {
            return PathBuf::from("./ontology_tools/package_management/tests/").join(filename);
        }
    }
    PathBuf::from("./tests/").join(filename)
}
