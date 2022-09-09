#![allow(dead_code)]
use camino::Utf8Path;
use colored::*;

use crate::{
    error::{CliError, FieldAccessError},
    manifest::FieldManifest,
};

pub trait Feedback {
    fn feedback(&self);
}

pub fn submission_failed(info: &str) {
    println!("\t{}", "Submission failed".red().bold(),);
    println!("\t{} {info}", "Info".yellow().bold(),);
    std::process::exit(0xFF);
}

pub fn login_failed(advice: &str) {
    println!("\t{}", "Login failed".red().bold(),);
    println!("\t{} {advice}", "Advice".yellow().bold(),);
    std::process::exit(0xFF);
}

pub fn submission_remote_linting_failed(failures: &[String]) {
    println!("\t{}", "Submission failed".red().bold(),);
    println!("\t{}", "Info".yellow().bold(),);
    for failure in failures {
        println!("\t  {}", failure);
    }
    std::process::exit(0xFF);
}

pub fn command_failed(info: &str) {
    println!("\t{}", "Command failed".red().bold(),);
    println!("\t{} {info}", "Info".yellow().bold(),);
    std::process::exit(0xFF);
}

pub fn dependency_resolution_failed(reason: &str) {
    println!("\t{}", "Failed to resolve dependencies".red().bold(),);
    println!("\t{} {reason}", "Reason".yellow().bold(),);
    std::process::exit(0xFF);
}

pub fn linting_failed() {
    println!("\t{}", "Linting failed".red().bold(),);
    println!(
        "\t{} Depending on the red lines in the linting output, update your field and try again.",
        "Advice".yellow().bold(),
    );
    std::process::exit(0xFF);
}

pub fn field_info(local_path: &Utf8Path) -> Result<(), CliError> {
    if !local_path.exists() {
        return Err(CliError::from(FieldAccessError::FailedToFindFieldAtPath {
            field_path: local_path.to_string(),
        }));
    }
    let contents = std::fs::read_to_string(&local_path).map_err(|_| {
        CliError::from(FieldAccessError::FailedToReadFieldAtPath {
            field_path: local_path.to_string(),
        })
    })?;

    let manifest = FieldManifest::new(contents).map_err(|_| {
        CliError::from(FieldAccessError::FailedToReadFieldManifest {
            field_path: local_path.to_string(),
        })
    })?;

    let full_name = manifest
        .field_namespace_and_name()
        .unwrap_or_else(|| "Not specified".to_owned().italic().to_string());
    let version = manifest
        .field_version()
        .unwrap_or_else(|| "Not specified".to_owned().italic().to_string());
    let dependencies = manifest.field_dependency_literals();

    println!("\t{} {full_name}", "Name".bold());
    println!("\t{} {version}", "Version".bold());
    println!("\t{} {local_path}", "Location".bold());
    if let Some(dependencies) = dependencies {
        println!("\t{}", "Requested Dependencies".bold());
        for dependency in &dependencies {
            println!("\t\t{dependency}");
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn info(info: &str) {
    println!("\t{} {info}", "Info".yellow().bold());
}

pub fn command_not_complete(advice: &str) {
    println!("\t{}", "Command is not complete".red().bold(),);
    println!("\t{} {advice}", "Advice".yellow().bold(),);
    std::process::exit(0xFF);
}

pub fn submission_lint_start() {
    println!(
        "\t{} the field before submission..",
        "Linting".green().bold(),
    );
}

pub fn lint_start(lint_set_name: &str) {
    println!();
    println!(
        "\t{} the provided field with {lint_set_name}..",
        "Linting".green().bold(),
    );
}

pub fn general_lint_start() {
    println!("\t{} the provided field..", "Linting".green().bold(),);
}

pub fn general_lint_success() {
    println!();
    println!("\t{} successful.", "Linting".green().bold(),);
}
pub fn general_update_success() {
    println!();
    println!("\t{} successful.", "Update".green().bold(),);
}
