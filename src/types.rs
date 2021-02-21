use std::fmt;

enum FileState {
    ADDED,
    MODIFIED,
    RENAMED,
    REMOVED,
    UNTRACKED,
    UNCHANGED,
    UNKNOWN,
}

pub struct ProjectFile {
    pub file_path: String,
    state: FileState,
}

impl ProjectFile {
    // Documentation: https://git-scm.com/docs/git-status
    pub fn from_line(line: &str) -> ProjectFile {
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
        ProjectFile { 
            file_path, 
            state,
        }
    }

    pub fn is_staged(&self) -> bool {
        match self.state {
            FileState::UNCHANGED    => true,
            _                       => false,
        }
    }

    pub fn is_not_staged(&self) -> bool {
        !self.is_staged()
    }
}

impl fmt::Display for ProjectFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.file_path)
    }
}

pub struct Commit {
    pub hash: String, 
    date: String,
    message_extract: String,
}

impl Commit {
    pub fn message_extract_width() -> usize {
        60
    }

    pub fn from_line(line: &str) -> Commit {
        let mut split_line = line.splitn(3, " | ");
        let hash: &str = split_line.next().unwrap();
        let date: &str = split_line.next().unwrap();
        let commit_message: &str = split_line.next().unwrap();
        let mut message_extract = String::from(commit_message.split("\n").next().unwrap());
        message_extract.truncate(Commit::message_extract_width());
        message_extract = String::from(message_extract.trim());
        if commit_message.len() > message_extract.len() {
            message_extract.push_str("...");
        }
        Commit {
            hash: String::from(hash),
            date: String::from(date),
            message_extract: String::from(message_extract),
        }
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} | {:^31} | {}", self.hash, self.date, self.message_extract)
    }
}

pub struct Branch {
    pub name: String, 
    pub is_checked_out: bool,
}

impl Branch {
    pub fn from_line(line: &str) -> Branch {
        let (branch_status, branch_name) = line.split_at(2);
        let name = String::from(branch_name.trim_start()); 
        let is_checked_out = branch_status.contains("*");
        Branch {
            name, 
            is_checked_out,
        }
    }

    pub fn new_branch_placeholder() -> Branch {
        Branch {
            name: String::from("[<create new branch>]"), 
            is_checked_out: false,
        }
    }
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", if self.is_checked_out { "*" } else { " " }, self.name)
    }
}

impl PartialEq for Branch {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct Action<'a> {
    name: String,
    callback: Box<dyn Fn() + 'a>
}

impl<'a> Action<'a> {
    pub fn new<C>(name: &str, callback_closure: C) -> Action<'a>
    where C: Fn() + 'a {
        Action {
            name: String::from(name),
            callback: Box::new(callback_closure),
        }
    }

    pub fn run_action(&self) {
        (self.callback)();
    }
}

impl fmt::Display for Action<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

