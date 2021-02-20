use crate::utils::run_command_capture_output;
use crate::types::{
    ProjectFile, 
    Commit,
    Branch,
};

pub fn get_git_root_dir() -> String {
    let command_output = run_command_capture_output(
        "git",
        &["rev-parse", "--show-toplevel"],
        "getting git repo root directory failed",
        None,
    );
    command_output.trim().to_string() // remove trailing newline from output before returning
}

pub fn get_modified_files() -> Vec<ProjectFile> {
    let command_output = run_command_capture_output(
        "git",
        &["status", "--porcelain"],
        "getting status of changed files in repo failed",
        Some(&get_git_root_dir()),
    );
    let lines: Vec<&str> = command_output.lines().collect();
    lines.into_iter().map(ProjectFile::from_line).collect()
}

pub fn get_commit_history() -> Vec<Commit> {
    let command_output = run_command_capture_output(
        "git",
        &["log", "--pretty=format:%h | %aD | %s"],
        "getting commit history failed",
        None,
    );
    let lines: Vec<&str> = command_output.lines().collect();
    lines.into_iter().map(Commit::from_line).collect()
}

pub fn get_branches() -> Vec<Branch> {
    let command_output = run_command_capture_output(
        "git",
        &["branch"],
        "getting branches failed",
        None,
    );
    let lines: Vec<&str> = command_output.lines().collect();
    let mut branches: Vec<Branch> = lines.into_iter().map(Branch::from_line).collect();
    branches.push(Branch::new_branch_placeholder());
    branches
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

