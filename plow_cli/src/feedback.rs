use colored::*;

pub fn submission_failed(info: &str) {
    println!("\t{}", "Submission failed".red().bold(),);
    println!("\t{} {info}", "Info".yellow().bold(),);
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

pub fn command_failed(advice: &str) {
    println!("\t{}", "Command failed".red().bold(),);
    println!("\t{} {advice}", "Advice".yellow().bold(),);
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

pub fn info(info: &str) {
    println!("\t{} {info}", "Info".yellow().bold());
}

pub fn command_not_complete(advice: &str) {
    println!("\t{}", "Command is not complete".red().bold(),);
    println!("\t{} {advice}", "Advice".yellow().bold(),);
    std::process::exit(0xFF);
}
