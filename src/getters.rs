use crate::utils::run_command_capture_output;
use crate::types::{
    ProjectFile, 
    Commit,
};

pub fn get_git_root_dir() -> String {
    let command_output = run_command_capture_output(
        "git",
        &["rev-parse", "--show-toplevel"],
        "getting git repo root directory failed",
    );
    command_output.trim().to_string() // remove trailing newline from output before returning
}

pub fn get_modified_files() -> Vec<ProjectFile> {
    let command_output = run_command_capture_output(
        "git",
        &["status", "--porcelain"],
        "getting status of changed files in repo failed",
    );
    let lines: Vec<&str> = command_output.split("\n").filter(|&s| !s.is_empty()).collect();
    lines.into_iter().map(ProjectFile::from_line).collect()
}

pub fn get_commit_history() -> Vec<Commit> {
    let command_output = run_command_capture_output(
        "git",
        &["log", "--pretty=format:%h | %aD | %s"],
        "getting commit history failed",
    );
    let lines: Vec<&str> = command_output.split("\n").collect();
    lines.into_iter().map(Commit::from_line).collect()
}

pub fn get_branches() -> Vec<String> {
    let command_output = run_command_capture_output(
        "git",
        &["branch"],
        "getting branches failed",
    );
    let lines: Vec<&str> = command_output.split("\n").filter(|&s| !s.is_empty()).collect();
    lines.into_iter().map(String::from).collect()
}

pub fn get_staged_files() -> Vec<ProjectFile> {
    get_modified_files().into_iter()
        .filter(ProjectFile::is_staged)
        .collect()
}

pub fn get_not_staged_files() -> Vec<ProjectFile> {
    get_modified_files().into_iter()
        .filter(ProjectFile::is_not_staged)
        .collect()
}

