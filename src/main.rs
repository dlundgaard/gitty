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

fn main() {
    let git_repo = GitRepo::new();

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
		   "status"     => git_repo.print_status(),
		   "stage"      => git_repo.staging_mode(),
		   "unstage"    => git_repo.unstaging_mode(),
		   "commit"     => git_repo.do_commit(),
		   "push"       => git_repo.do_push(),
		   "pull"       => git_repo.do_pull(),
		   "exit"       => { println!("\nThanks for stopping by!"); exit(0) },
		   _            => panic!("Unknown action"),
		}
	}
}

struct GitRepo {
    repo_root_dir: String,
}

impl GitRepo {
    fn new() -> GitRepo {
        let command_output = Command::new("git")
            .args(&["rev-parse", "--show-toplevel"])
            .output()
            .expect("git command failed");
        let mut output_as_string = String::from_utf8_lossy(&command_output.stdout).to_string();
        output_as_string.pop(); // remove trailing newline from output
        GitRepo { repo_root_dir: output_as_string.to_owned() }
    }

    fn get_changed_files(&self) -> Vec<ProjectFile> {
        let command_output = Command::new("git")
            .args(&["status", "--porcelain"])
            .output()
            .expect("git command failed");
        let output_as_string = String::from_utf8_lossy(&command_output.stdout);
        let lines: Vec<&str> = output_as_string.split("\n").filter(|&s| !s.is_empty()).collect();
        lines.into_iter().map(ProjectFile::from_line).collect()
    }

    fn get_staged_files(&self) -> Vec<ProjectFile> {
        self.get_changed_files().into_iter()
            .filter(ProjectFile::is_staged)
            .collect()
    }

    fn get_not_staged_files(&self) -> Vec<ProjectFile> {
        self.get_changed_files().into_iter()
            .filter(ProjectFile::is_not_staged)
            .collect()
    }

    fn clear_lines(&self, amount_lines_to_clear: usize) {
        Term::stdout().clear_last_lines(amount_lines_to_clear).unwrap();
    }

    fn confirm_return(&self) {
        Term::stdout().write_str("\nPress any key to go back ").unwrap();
        Term::stdout().read_key().unwrap();
        Term::stdout().write_line("").unwrap();
        self.clear_lines(2);
    }

    fn print_status(&self) {
        let command_output = Command::new("git")
            .args(&["status"])
            .current_dir(&self.repo_root_dir)
            .output()
            .expect("git command failed");
        let output_as_string = String::from_utf8_lossy(&command_output.stdout);
        let amount_lines = output_as_string.matches("\n").count();
        Term::stdout().write_str(&output_as_string).unwrap();
        self.confirm_return();
        self.clear_lines(amount_lines);
    }

    fn staging_mode(&self) {
        let unstaged_files = self.get_not_staged_files();
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
                    .current_dir(&self.repo_root_dir)
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
        self.clear_lines(amount_lines);
    }

    fn unstaging_mode(&self) {
        let staged_files = self.get_staged_files();
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
                    .current_dir(&self.repo_root_dir)
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
        self.clear_lines(amount_lines);
    }

    fn do_commit(&self) {
        let staged_files = self.get_staged_files();
        let unstaged_files = self.get_not_staged_files();
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
                    .current_dir(&self.repo_root_dir)
                    .output()
                    .expect("git command failed");
                let output_as_string = String::from_utf8_lossy(&command_output.stdout);
                println!("{}", output_as_string);
            }
        } else {
            // TODO 
        }
    }

    fn do_push(&self) {
    }

    fn do_pull(&self) {
    }
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
