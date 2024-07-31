use clap::Command;
use colored::*;
use inquire::Select;
use fs_extra::dir::{copy, CopyOptions};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
enum SetupError {
    #[error("Failed to get current directory")]
    CurrentDirError(#[from] std::io::Error),

    #[error("Failed to copy template files for {0}")]
    CopyError(String, #[source] fs_extra::error::Error),

    #[error("Failed to read input")]
    InputError(#[from] inquire::error::InquireError),
}

type Result<T> = std::result::Result<T, SetupError>;

fn setup_project(language: &str) -> Result<()> {
    println!("Initializing a new {} project...", language);
    let project_name = format!("{}_template", language.to_lowercase());
    let project_path = std::env::current_dir()?;

    match language {
        "rust" => {
            println!("Creating Rust project in directory '{}'", project_name);
            let template_path = PathBuf::from("src/templates/htmx-rust");

            let mut options = CopyOptions::new();
            options.copy_inside = true;

            copy(&template_path, &project_path, &options)
                .map_err(|e| SetupError::CopyError("Rust".to_string(), e))?;

            println!("Rust project setup complete!");
        },
        "go" => {
            println!("Creating Go project in directory '{}'", project_name);
            let template_path = PathBuf::from("src/templates/htmx-go");

            let mut options = CopyOptions::new();
            options.copy_inside = true;

            copy(&template_path, &project_path, &options)
                .map_err(|e| SetupError::CopyError("Go".to_string(), e))?;

            println!("Go project setup complete!");
        },
        "typescript" => {
            println!("Creating TypeScript project in directory '{}'", project_name);
            let template_path = PathBuf::from("src/templates/htmx-typescript");

            let mut options = CopyOptions::new();
            options.copy_inside = true;

            copy(&template_path, &project_path, &options)
                .map_err(|e| SetupError::CopyError("TypeScript".to_string(), e))?;

            println!("TypeScript project setup complete!");
        },
        _ => unreachable!(),
    }

    println!("Project setup complete for {}!", language);
    Ok(())
}

fn main() -> Result<()> {
    Command::new("htmx-starter")
        .version("0.0.1")
        .author("reed11tim@gmail.com")
        .about("Initializes a new project with Rust, Go, and TypeScript")
        .get_matches();

    let rust_option = "Rust".bold().truecolor(184, 115, 51).to_string();
    let go_option = "Go".bold().truecolor(0, 173, 216).to_string();
    let typescript_option = "TypeScript".bold().truecolor(0, 122, 204).to_string();
    let languages = vec![rust_option, go_option, typescript_option];

    let selection = Select::new("Select the language for your project:", languages)
        .prompt()?;

    match selection.as_str() {
        s if s.contains("Rust") => setup_project("rust")?,
        s if s.contains("Go") => setup_project("go")?,
        s if s.contains("TypeScript") => setup_project("typescript")?,
        _ => unreachable!(),
    }

    Ok(())
}