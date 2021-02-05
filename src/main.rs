use std::process::{
    exit, 
    Command,
};
use dialoguer::{
    Select, 
    MultiSelect,
    Confirm,
    theme::ColorfulTheme, 
};
use console::Term;

fn main() {
    let files_in_repo = get_changed_files();

	loop {
		let actions = [
			"status",
			"stage",
			"unstage",
			"commit",
			"push",
			"pull",
			"exit",
		];
		let selected = Select::new()
			.with_prompt("What do you want to do?")
			.items(&actions[..])
			.default(0)
			.interact()
			.unwrap();
		let selected_action = actions[selected];
		match selected_action {
		   "status"     => print_status(),
		   "stage"      => staging_mode(),
		   "unstage"    => unstaging_mode(),
		   "commit"     => do_commit(),
		   "push"       => do_push(),
		   "pull"       => do_pull(),
		   "exit"       => { println!("Thanks for stopping by!"); exit(0) },
		   _            => panic!("Unknown action"),
		}
		println!("You selected: {}", actions[selected]);
	}
}

fn get_changed_files() -> Vec<ProjectFile> {
    let command_name = "git";
    let args = ["status", "--porcelain"];
    let command_output = Command::new(command_name).args(&args).output().expect("git command failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
    //println!("{:?}", command_output);
    //println!("{}", output_as_string);

    let lines: Vec<&str> = output_as_string.split("\n").filter(|&s| !s.is_empty()).collect();
    println!("{:?}", &lines);

    let project_files: Vec<ProjectFile> = lines.into_iter().map(ProjectFile::from_line).collect();
    return project_files;
}

fn get_staged_files() -> Vec<ProjectFile> {
    get_changed_files().into_iter()
        //.map(|e| {println!("{:?}", e); e})
		.filter(ProjectFile::is_staged)
        .collect()
}

fn get_not_staged_files() -> Vec<ProjectFile> {
    get_changed_files().into_iter()
        //.map(|e| {println!("{:?}", e); e})
		.filter(ProjectFile::is_not_staged)
        .collect()
}

fn clear_output(amount_lines_to_clear: usize) {
    Term::stdout().clear_last_lines(amount_lines_to_clear).unwrap();
}

fn print_status() {
    let command_output = Command::new("git").args(&["status"]).output().expect("git command failed");
    let output_as_string = String::from_utf8_lossy(&command_output.stdout);
	let amount_lines = output_as_string.matches("\n").count();
    Term::stdout().write_str(&output_as_string).unwrap();
    Term::stdout().read_key().unwrap();
    clear_output(amount_lines);
}

fn staging_mode() {
	let unstaged_files = get_not_staged_files();
	let unstaged_files_names: Vec<String> = unstaged_files.into_iter().map(|pf| pf.file_path).collect();
    //println!("{:?}", unstaged_files_names);
	if !unstaged_files_names.is_empty() { 
        let selections = MultiSelect::new()
            .with_prompt("Which files should be staged?")
            .items(&unstaged_files_names[..])
            .interact()
            .unwrap();
        for selected in selections {
            println!("{} added to staging area", unstaged_files_names[selected]);
        }
    } else {
        println!("No unstaged files");
        clear_output(1);
    }
}

fn unstaging_mode() {
	let staged_files = get_staged_files();
	let staged_files_names: Vec<String> = staged_files.into_iter().map(|pf| pf.file_path).collect();
	if !staged_files_names.is_empty() { 
        let selections = MultiSelect::new()
            .with_prompt("Which files should be unstaged?")
            .items(&staged_files_names[..])
            .interact()
            .unwrap();
        for selected in selections {
            println!("{} removed from staging area", staged_files_names[selected]);
        }
    } else {
        println!("No staged files");
        clear_output(1);
    }
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
            // TODO
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
