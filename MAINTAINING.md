# Publishing a new version

- Create a new branch
- Bump the version of the crate(s) you want to release (via [cargo-release](https://github.com/crate-ci/cargo-release))(:
  - Dry run with e.g. `cargo release version --package plow_graphify patch`
  - Actually execute it by appending the `--execute` flag
- Push the branch
- Merge the branch to `main` (the release workflow there should take care of publishing the crate)