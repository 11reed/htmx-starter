use clap::Command;
use inquire::Select;
use std::fs;
use fs_extra::dir::{copy, CopyOptions};
use std::path::PathBuf;
use std::process::Command as StdCommand;


fn main() {
    // Set up the command line interface metadata
    Command::new("htmx-starter")
        .version("0.0.1")
        .author("reed11tim@gmail.com")
        .about("Initializes a new project with Rust, Go, and TypeScript")
        .get_matches();  // This call initializes the command, but we do not store its result

    // Prompt the user to select a language for the project
    let languages = vec!["Rust", "Go", "TypeScript: Not currently supported"];
    let selection = Select::new("Select the language for your project:", languages)
        .prompt()
        .expect("Failed to read input");

    match selection {
        "Rust" => setup_project("rust"),
        "Go" => setup_project("go"),
        "TypeScript" => setup_project("typescript"),
        _ => unreachable!(), // This guards against unexpected input
    }
}

fn setup_project(language: &str) {
    println!("Initializing a new {} project...", language);
    let project_name = format!("{}_template", language.to_lowercase());
    fs::create_dir_all(&project_name).expect("Failed to create project directory");

    match language {
        "rust" => {
            println!("Creating Rust project in directory '{}'", project_name);
            let template_path = PathBuf::from("src/templates/htmx-rust");
            let project_path = PathBuf::from(&project_name);

            // Set copy options
            let mut options = CopyOptions::new();
            options.copy_inside = true;

            // Copy the template directory to the new project directory
            copy(template_path, project_path, &options).expect("Failed to copy template files");

            println!("Rust project setup complete!");
        },
        "go" => {
            println!("Creating Go project in directory '{}'", project_name);
            // Change to the new project directory and run go mod init
            std::env::set_current_dir(&project_name).expect("Failed to change directory");
            StdCommand::new("go")
                .arg("mod")
                .arg("init")
                .arg(project_name)
                .status()
                .expect("Failed to initialize Go module");
            // Optionally, create a main.go file
            let main_go_content = r#"
                    package main

                    import "fmt"

                    func main() {
                        fmt.Println("Hello, world!")
                    }
                    "#;
            fs::write("main.go", main_go_content).expect("Failed to create main.go file");
        },
        "typescript" => {
            println!("Creating TypeScript project in directory '{}'", project_name);
            // Change to the new project directory and initialize npm
            std::env::set_current_dir(&project_name).expect("Failed to change directory");
            StdCommand::new("npm")
                .arg("init")
                .arg("-y")
                .status()
                .expect("Failed to initialize npm project");
            // Install TypeScript and its dependencies
            StdCommand::new("npm")
                .args([
                    "install", 
                    "--save-dev", 
                    "typescript", 
                    "@types/node",
                    "@types/express",
                    "nodemon"
                    ])
                .status()
                .expect("Failed to install TypeScript dev dependencies");

            StdCommand::new("npm")
            .args([
                "install", 
                "express",
                "@preact/signals-core"
                ])
            .status()
            .expect("Failed to install TypeScript dependencies");
            // Create a tsconfig.json file
            let tsconfig = r#"{
                "compilerOptions": {
                    "target": "es6",
                    "module": "commonjs",
                    "strict": true,
                    "esModuleInterop": true,
                    "skipLibCheck": true,
                    "forceConsistentCasingInFileNames": true
                }
            }"#;
            fs::write("tsconfig.json", tsconfig)
                .expect("Failed to create tsconfig.json");
        },
        _ => unreachable!(),
    }

    println!("Project setup complete for {}!", language);
}
