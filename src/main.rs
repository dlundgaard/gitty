use std::process::Command;
use dialoguer::{theme::ColorfulTheme, Select, MultiSelect};

fn main() {
    let files_in_repo = get_files_in_repo();

    let selectables = &[
        "status",
        "add to staged",
        "commit",
        "push",
        "pull",
    ];
    let selected = Select::with_theme(&ColorfulTheme::default())
        //.with_prompt("What do you want to do?")
        .items(&selectables [..])
        .default(0)
        .interact()
        .unwrap();

    println!("You selected: {}", selectables[selected]);
}

fn get_files_in_repo() -> Vec<String> {
    let command_name = "git";
    let args = ["status", "--porcelain"];
    let command_output = Command::new(command_name).args(&args).output().expect("git command failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    println!("{:?}", command_output);
    println!("{}", output_as_string);

    let lines: Vec<&str> = output_as_string.split("\n").filter(|&s| !s.is_empty()).collect();
    println!("{:?}", &lines);
    let project_files: Vec<ProjectFile> = lines.into_iter().map(ProjectFile::from_line).collect();
    println!("ProjectFiles:");
    for pf in project_files {
        println!("  {:?}", pf);
    }

    let result = Vec::new();
    return result;
}

#[derive(Debug)]
enum FileState {
    ADDED,
    MODIFIED,
    RENAMED,
    REMOVED,
    UNTRACKED,
    UNKNOWN,
}

#[derive(Debug)]
struct ProjectFile {
    file_path: String,
    state: FileState,
}

impl ProjectFile {
    fn from_line(line: &str) -> ProjectFile {
        let mut split_line = line.trim_start().splitn(2, " ");
        let state_code = split_line.next().unwrap_or("-");
        let state = match state_code {
            "A"  => FileState::ADDED,
            "M"  => FileState::MODIFIED,
            "R"  => FileState::RENAMED,
            "D"  => FileState::REMOVED,
            "??" => FileState::UNTRACKED,
            _    => FileState::UNKNOWN,
        };
        let file_path = String::from(split_line.next().unwrap());
        ProjectFile { file_path, state }
    }
}
