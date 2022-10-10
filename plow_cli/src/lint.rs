use colored::*;
use plow_linter::lint::LintResult;
use plow_linter::lints::*;
use plow_linter::Linter;

use crate::error::CliError;

use crate::error::LintSubcommandError::*;
use crate::feedback::{general_lint_success, lint_start, Feedback};

pub struct SuccessfulLint;
impl Feedback for SuccessfulLint {
    fn feedback(&self) {
        general_lint_success();
    }
}

pub fn lint_file(field_path: &str, lints: Vec<LintSet>) -> Result<(), CliError> {
    let field_contents = std::fs::read_to_string(field_path).map_err(|err| FailedToReadField {
        field_path: field_path.to_owned(),
        details: err.to_string(),
    })?;

    let mut contains_failures = false;
    let mut linter = Linter::try_from(field_contents.as_ref()).map_err(|_| FailedToParseField {
        field_path: field_path.to_owned(),
    })?;

    for lint_set in lints {
        let set_id = lint_set.id;
        let set_name = linter.add_lint_set(lint_set);
        lint_start(&set_name);

        let results = linter.run_lint_set(set_id);
        for result in results {
            use LintResult::*;
            match result {
                Success(message) => {
                    println!("\t\t{}", message.green());
                }
                Warning(messages) => {
                    for message in messages {
                        println!("\t\t{}", message.yellow());
                    }
                }
                Failure(messages) => {
                    for message in messages {
                        println!("\t\t{}", message.red());
                    }
                    contains_failures = true;
                }
            }
        }
    }

    if contains_failures {
        return Err(LintsContainFailuresOpaque.into());
    }
    Ok(())
}

pub fn lint_file_fail_on_failure(field_path: &str, lints: LintSet) -> Result<(), CliError> {
    let field_contents = std::fs::read_to_string(field_path).map_err(|err| FailedToReadField {
        field_path: field_path.to_owned(),
        details: err.to_string(),
    })?;

    let mut linter = Linter::try_from(field_contents.as_ref()).map_err(|_| FailedToParseField {
        field_path: field_path.to_owned(),
    })?;

    linter.add_lint_set(lints);

    if linter.run_lints_check_if_contains_any_failure() {
        return Err(SingleLintContainsFailure {
            field_path: field_path.to_owned(),
        }
        .into());
    }

    Ok(())
}
