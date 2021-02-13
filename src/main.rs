use std::fmt;
use std::process::{
    exit, 
    Command,
};
use dialoguer::{
    Select, 
    MultiSelect,
    Confirm,
};
use console::Term;

// TODO

fn run_command(command_name: &str, command_args: &[&str], error_message: &str) {
    let mut command_handle = Command::new(command_name)
        .args(command_args)
        .spawn()
        .expect(error_message);
    command_handle.wait().expect(error_message);
}

fn run_command_in_dir(in_directory: &str, command_name: &str, command_args: &[&str], error_message: &str) {
    let mut command_handle = Command::new(command_name)
        .args(command_args)
        .current_dir(in_directory)
        .spawn()
        .expect(error_message);
    command_handle.wait().expect(error_message);
}

fn get_git_root_dir() -> String {
    let command_output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .expect("getting git repo root directory failed");
    let mut output_as_string = String::from_utf8_lossy(&command_output.stdout).to_string();
    output_as_string.pop(); // remove trailing newline from output
    output_as_string.to_owned()
}

fn get_modified_files() -> Vec<ProjectFile> {
    let command_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .expect("getting status of changed files in repo failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    let lines: Vec<&str> = output_as_string.split("\n").filter(|&s| !s.is_empty()).collect();
    lines.into_iter().map(ProjectFile::from_line).collect()
}

fn get_commit_history() -> Vec<Commit> {
    let command_output = Command::new("git")
        .args(&["log", "--pretty=format:%h | %aD | xx"])
        //.args(&["log", "--pretty=format:%h | %aD | %s"])
        .output()
        .expect("getting commit history failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    let lines: Vec<&str> = output_as_string.split("\n").collect();
    lines.into_iter().map(Commit::from_line).collect()

}

fn get_staged_files() -> Vec<ProjectFile> {
    get_modified_files().into_iter()
        .filter(ProjectFile::is_staged)
        .collect()
}

fn get_not_staged_files() -> Vec<ProjectFile> {
    get_modified_files().into_iter()
        .filter(ProjectFile::is_not_staged)
        .collect()
}

fn clear_lines(amount_lines_to_clear: usize) {
    Term::stdout().clear_last_lines(amount_lines_to_clear).unwrap();
}

fn confirm_return() {
    Term::stdout().write_str("\nPress any key to go back ").unwrap();
    Term::stdout().read_key().unwrap();
    Term::stdout().write_line("").unwrap();
    clear_lines(2);
}

fn show_status() {
    run_command(
        "git", 
        &["status"], 
        "git status failed"
    );
    confirm_return();
}

fn show_log() {
    run_command(
        "git", 
        &["--no-pager", "log", "--reverse"], 
        "git log failed"
    );
    confirm_return();
}

fn show_diff() {
    run_command(
        "git", 
        &["--no-pager", "diff", "--reverse"], 
        "git diff failed"
    );
    confirm_return();
}

fn staging_mode(repo_root_dir: &str) {
    let unstaged_files = get_not_staged_files();
    if unstaged_files.is_empty() { 
        println!("There are no unstaged files");
        confirm_return();
    } else {
        let selections = MultiSelect::new()
            .with_prompt("Which files should be staged?")
            .items(&unstaged_files)
            .interact()
            .unwrap();
        for selected in selections {
            let file_to_be_staged = &unstaged_files[selected].file_path;
            run_command_in_dir(
                &repo_root_dir,
                "git", 
                &["add", file_to_be_staged], 
                &format!("git add \"{}\" failed", file_to_be_staged), 
            );
            println!("{} added to staging area", file_to_be_staged);
        }
    }
}

fn unstaging_mode(repo_root_dir: &str) {
    let staged_files = get_staged_files();
    if staged_files.is_empty() { 
        println!("There are no staged files");
        confirm_return();
    } else {
        let selections = MultiSelect::new()
            .with_prompt("Which files should be unstaged?")
            .items(&staged_files)
            .interact()
            .unwrap();
        for selected in selections {
            let file_to_be_unstaged = &staged_files[selected].file_path;
            run_command_in_dir(
                &repo_root_dir,
                "git", 
                &["reset", file_to_be_unstaged], 
                &format!("git reset \"{}\" failed", file_to_be_unstaged), 
            );
            println!("{} removed from staging area", file_to_be_unstaged);
        }
    }
}

fn do_commit() {
    let staged_files = get_staged_files();
    let unstaged_files = get_not_staged_files();
    if staged_files.is_empty() {
        println!("There are no staged files");
        return;
    } else if unstaged_files.len() > 0 {
        let commit_despite_unstaged_file = Confirm::new()
            .with_prompt("There are modified files that have not been staged. Do you want to commit anyway?")
            .interact()
            .unwrap();
        if !commit_despite_unstaged_file { 
            return;
        }
    } 
    run_command(
        "git", 
        &["commit"], 
        "git commit failed"
    );
}

fn do_push() {
    run_command(
        "git", 
        &["push"], 
        "git push failed"
    );
    confirm_return();
}

fn do_pull() {
    run_command(
        "git", 
        &["pull"], 
        "git pull failed"
    );
    confirm_return();
}

fn checkout_mode() {
    let all_commits = get_commit_history();
    let selected_opt = Select::new()
        .with_prompt("Which commit would you like to checkout?")
        .items(&all_commits)
        .default(0)
        .interact_opt()
        .unwrap();
    if let Some(selected) = selected_opt {
        let selected_commit = &all_commits[selected];
        run_command(
            "git", 
            &["checkout", &selected_commit.hash], 
            &format!("git checkout {} failed", selected_commit.hash)
        );
    }
}

#[derive(Debug)]
enum FileState {
    ADDED,
    MODIFIED,
    RENAMED,
    REMOVED,
    UNTRACKED,
    UNCHANGED,
    UNKNOWN,
}

#[derive(Debug)]
struct ProjectFile {
    file_path: String,
    state: FileState,
}

impl ProjectFile {
    // Documentation: https://git-scm.com/docs/git-status
    fn from_line(line: &str) -> ProjectFile {
        let (state_code, file_path_raw) = line.split_at(2);
        let mut state_code_chars = state_code.chars();
        state_code_chars.next();
        let unstaged_state_code = state_code_chars.next().unwrap_or('_');
        let state = match unstaged_state_code {
            'A' => FileState::ADDED,
            'M' => FileState::MODIFIED,
            'R' => FileState::RENAMED,
            'D' => FileState::REMOVED,
            '?' => FileState::UNTRACKED,
            ' ' => FileState::UNCHANGED,
             _  => FileState::UNKNOWN,
        };
        let file_path = String::from(file_path_raw.trim());
        ProjectFile { file_path, state }
    }

    fn is_staged(&self) -> bool {
        match self.state {
            FileState::UNCHANGED    => true,
            _                       => false,
        }
    }

    fn is_not_staged(&self) -> bool {
        !self.is_staged()
    }
}

impl fmt::Display for ProjectFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.file_path)
    }
}

struct Commit {
    hash: String, 
    date: String,
    commit_message: String,
}

impl Commit {
    fn from_line(line: &str) -> Commit {
        let mut split_line = line.split(" | ").into_iter();
        let hash: &str = split_line.next().unwrap();
        let date: &str = split_line.next().unwrap();
        let commit_message: &str = split_line.next().unwrap();
        Commit {
            hash: String::from(hash),
            date: String::from(date),
            commit_message: String::from(commit_message),
        }
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} | {} | {}", self.hash, self.date, self.commit_message)
    }
}

struct Action<'a> {
    name: String,
    callback: Box<dyn Fn() + 'a>
}

impl<'a> Action<'a> {
    fn new<C>(name: &str, callback_closure: C) -> Action<'a>
    where C: Fn() + 'a {
        Action {
            name: String::from(name),
            callback: Box::new(callback_closure),
        }
    }

    fn run_action(&self) {
        (self.callback)();
    }
}

impl fmt::Display for Action<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn exit_gracefully() {
    println!("\nThanks for stopping by!"); 
    exit(0);
}

fn main() {
    let git_root_dir = get_git_root_dir();
    let mut actions: Vec<Action> = Vec::new();

    actions.push(
        Action::new("status", || show_status())
    );
    actions.push(
        Action::new("log", || show_log())
    );
    actions.push(
        Action::new("diff", || show_diff())
    );
    actions.push(
        Action::new("stage", || staging_mode(&git_root_dir))
    );
    actions.push(
        Action::new("unstage", || unstaging_mode(&git_root_dir))
    );
    actions.push(
        Action::new("commit", || do_commit())
    );
    actions.push(
        Action::new("checkout", || checkout_mode())
    );
    actions.push(
        Action::new("push", || do_push())
    );
    actions.push(
        Action::new("pull", || do_pull())
    );
    actions.push(
        Action::new("exit", || exit_gracefully())
    );

    let mut last_selected = 0;

	loop {
		let selected = Select::new()
			.with_prompt("What would you like to do?")
			.items(&actions)
			.default(last_selected)
			.interact()
			.unwrap();
        last_selected = selected;
        let selected_action = &actions[selected];
        selected_action.run_action();
	}
}

