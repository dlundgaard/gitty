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
// remove all println's and use Term.write_line
// error handling
// figure out getting colored terminal output when collecting from git status

fn get_git_root_dir() -> String {
    let command_output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output()
        .expect("git command failed");
    let mut output_as_string = String::from_utf8_lossy(&command_output.stdout).to_string();
    output_as_string.pop(); // remove trailing newline from output
    output_as_string.to_owned()
}

fn get_changed_files() -> Vec<ProjectFile> {
    let command_output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .expect("git command failed");
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

fn print_status() {
    let command_output = Command::new("git")
        .args(&["status"])
        .output()
        .expect("git command failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    let amount_lines = output_as_string.matches("\n").count();
    Term::stdout().write_str(&output_as_string).unwrap();
    confirm_return();
    clear_lines(amount_lines);
}

fn staging_mode(repo_root_dir: &str) {
    let unstaged_files = get_not_staged_files();
    let unstaged_files_names: Vec<String> = unstaged_files.into_iter().map(|pf| pf.file_path).collect();
    let mut amount_lines = 0;
    if !unstaged_files_names.is_empty() { 
        let selections = MultiSelect::new()
            .with_prompt("Which files should be staged?")
            .items(&unstaged_files_names[..])
            .interact()
            .unwrap();
        amount_lines += selections.len() + 1;
        for selected in selections {
            let command_output = Command::new("git")
                .args(&["add", &unstaged_files_names[selected]])
                .current_dir(&repo_root_dir)
                .output()
                .expect("git command failed");
            let output_as_string = String::from_utf8_lossy(&command_output.stdout);
            println!("out {:?}", command_output);
            amount_lines += output_as_string.matches("\n").count();
            Term::stdout().write_str(&output_as_string).unwrap();
            println!("{} added to staging area", unstaged_files_names[selected]);
        }
    } else {
        println!("No unstaged files");
        amount_lines += 1;
    }
    clear_lines(amount_lines);
}

fn unstaging_mode(repo_root_dir: &str) {
    let staged_files = get_staged_files();
    let staged_files_names: Vec<String> = staged_files.into_iter().map(|pf| pf.file_path).collect();
    let mut amount_lines = 0;
    if !staged_files_names.is_empty() { 
        let selections = MultiSelect::new()
            .with_prompt("Which files should be unstaged?")
            .items(&staged_files_names[..])
            .interact()
            .unwrap();
        amount_lines += selections.len() + 1;
        for selected in selections {
            let command_output = Command::new("git")
                .args(&["reset", &staged_files_names[selected]])
                .current_dir(&repo_root_dir)
                .output()
                .expect("git command failed");
            let output_as_string = String::from_utf8_lossy(&command_output.stdout);
            amount_lines += output_as_string.matches("\n").count();
            Term::stdout().write_str(&output_as_string).unwrap();
            println!("{} removed from staging area", staged_files_names[selected]);
        }
    } else {
        println!("No staged files");
        amount_lines += 1;
    }
    clear_lines(amount_lines);
}

fn do_commit() {
    let staged_files = get_staged_files();
    let unstaged_files = get_not_staged_files();
    if staged_files.is_empty() {
        println!("There are no staged files");
    } else if unstaged_files.len() > 0 {
        if Confirm::new()
            .with_prompt("There are unstaged files. Do you want to continue?")
            .interact()
            .unwrap() 
        { 
            let command_output = Command::new("git")
                .args(&["commit"])
                .output()
                .expect("git command failed");
            let output_as_string = String::from_utf8_lossy(&command_output.stdout);
            println!("{}", output_as_string);
        }
    } else {
        // TODO 
    }
}

fn do_push() {
}

fn do_pull() {
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
    fn new<F>(name: &str, callback_closure: F) -> Action<'a>
    where F: Fn() + 'a {
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
        Action::new("status", || print_status())
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

	loop {
		let selected = Select::new()
			.with_prompt("What do you want to do?")
			.items(&actions)
			.default(0)
			.interact()
			.unwrap();
        let selected_action = &actions[selected];
        selected_action.run_action();
	}
}

