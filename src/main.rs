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
// fix bug when file is both staged and has unstaged updates at the same time
// remove all println's and use Term.write_line
// error handling
// figure out getting colored terminal output when collecting from git status

fn get_git_root_dir() -> String {
    let command_output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .expect("getting git repo root directory failed");
    let mut output_as_string = String::from_utf8_lossy(&command_output.stdout).to_string();
    output_as_string.pop(); // remove trailing newline from output
    output_as_string.to_owned()
}

fn get_changed_files() -> Vec<ProjectFile> {
    let command_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .expect("getting status of changed files in repo failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    let lines: Vec<&str> = output_as_string.split("\n").filter(|&s| !s.is_empty()).collect();
    lines.into_iter().map(ProjectFile::from_line).collect()
}

fn get_staged_files() -> Vec<ProjectFile> {
    get_changed_files().into_iter()
        .filter(ProjectFile::is_staged)
        .collect()
}

fn get_not_staged_files() -> Vec<ProjectFile> {
    get_changed_files().into_iter()
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
    let command_output = Command::new("git")
        .args(&["status"])
        .output()
        .expect("git status failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    Term::stdout().write_str(&output_as_string).unwrap();
    confirm_return();
}

fn show_log() {
    let command_handle = Command::new("git")
        .args(&["--no-pager", "log"])
        .output()
        .expect("git log failed");
    let output_as_string = String::from_utf8_lossy(&command_handle.stdout);
    Term::stdout().write_str(&output_as_string).unwrap();
    confirm_return();
}

fn show_diff() {
    let command_output = Command::new("git")
        .args(&["--no-pager", "diff"])
        .output()
        .expect("git diff failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    Term::stdout().write_str(&output_as_string).unwrap();
    confirm_return();
}

fn staging_mode(repo_root_dir: &str) {
    let unstaged_files = get_not_staged_files();
    let unstaged_files_names: Vec<String> = unstaged_files.into_iter().map(|pf| pf.file_path).collect();
    if !unstaged_files_names.is_empty() { 
        let selections = MultiSelect::new()
            .with_prompt("Which files should be staged?")
            .items(&unstaged_files_names[..])
            .interact()
            .unwrap();
        for selected in selections {
            let file_to_be_staged = &unstaged_files_names[selected];
            let command_output = Command::new("git")
                .args(&["add", file_to_be_staged])
                .current_dir(&repo_root_dir)
                .output()
                .expect("git add failed");
            let output_as_string = String::from_utf8_lossy(&command_output.stdout);
            println!("out {:?}", command_output);
            Term::stdout().write_str(&output_as_string).unwrap();
            println!("{} added to staging area", unstaged_files_names[selected]);
        }
    } else {
        println!("There are unstaged files");
        confirm_return();
    }
}

fn unstaging_mode(repo_root_dir: &str) {
    let staged_files = get_staged_files();
    let staged_files_names: Vec<String> = staged_files.into_iter().map(|pf| pf.file_path).collect();
    if !staged_files_names.is_empty() { 
        let selections = MultiSelect::new()
            .with_prompt("Which files should be unstaged?")
            .items(&staged_files_names[..])
            .interact()
            .unwrap();
        for selected in selections {
            let file_to_be_unstaged = &staged_files_names[selected];
            let command_output = Command::new("git")
                .args(&["reset", file_to_be_unstaged])
                .current_dir(&repo_root_dir)
                .output()
                .expect("git reset failed");
            let output_as_string = String::from_utf8_lossy(&command_output.stdout);
            Term::stdout().write_str(&output_as_string).unwrap();
            println!("{} removed from staging area", staged_files_names[selected]);
        }
    } else {
        println!("There are no staged files");
        confirm_return();
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
    let mut command_handle = Command::new("git")
        .args(&["commit"])
        .spawn()
        .expect("git commit failed");
    command_handle.wait().expect("git commit failed");
}

fn do_push() {
    let command_handle = Command::new("git")
        .args(&["push"])
        .spawn()
        .expect("git push failed");
    let command_output = command_handle.wait_with_output().expect("git push failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    Term::stdout().write_str(&output_as_string).unwrap();
    confirm_return();
}

fn do_pull() {
    let mut command_handle = Command::new("git")
        .args(&["pull"])
        .spawn()
        .expect("git pull failed");
    command_handle.wait().expect("git pull failed");
    let command_output = command_handle.wait_with_output().expect("git pull failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    Term::stdout().write_str(&output_as_string).unwrap();
    confirm_return();
}

#[derive(Debug)]
enum FileState {
    ADDED,
    MODIFIED,
    RENAMED,
    REMOVED,
    UNTRACKED,
    UNSTAGED,
    UNKNOWN,
}

#[derive(Debug)]
struct ProjectFile {
    file_path: String,
    state: FileState,
}

impl ProjectFile {
    fn from_line(line: &str) -> ProjectFile {
        let (state_code, file_path_raw) = line.split_at(2);
        let state_char = state_code.chars().next().unwrap_or('X');
        let state = match state_char {
            'A' => FileState::ADDED,
            'M' => FileState::MODIFIED,
            'R' => FileState::RENAMED,
            'D' => FileState::REMOVED,
            '?' => FileState::UNTRACKED,
            ' ' => FileState::UNSTAGED,
             _  => FileState::UNKNOWN,
        };
        let file_path = String::from(file_path_raw.trim());
        ProjectFile { file_path, state }
    }

    fn is_staged(&self) -> bool {
        match self.state {
            FileState::UNSTAGED     => false,
            FileState::UNTRACKED    => false,
            _                       => true,
        }
    }

    fn is_not_staged(&self) -> bool {
        !self.is_staged()
    }
}

fn exit_gracefully() {
    println!("\nThanks for stopping by!"); 
    exit(0);
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

