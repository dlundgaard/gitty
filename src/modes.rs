use dialoguer::{
    Select, 
    MultiSelect,
    Confirm,
    Input,
};
use crate::utils;
use crate::getters;
use crate::types::{
    Action,
    Branch,
    ProjectFile,
};

pub fn select_command_mode() {
    let git_root_dir = getters::get_git_root_dir();
    let actions: Vec<Action> = vec![
        Action::new("status", || show_status()),
        Action::new("log", || show_log()),
        Action::new("diff", || show_diff()),
        Action::new("staging", || staging_mode(&git_root_dir)),
        Action::new("commit", || do_commit()),
        Action::new("checkout", || checkout_mode()),
        Action::new("branch", || branch_mode()),
        Action::new("push", || do_push()),
        Action::new("pull", || do_pull()),
        Action::new("exit", || utils::exit_gracefully()),
    ];

    let mut last_selected = 0;

	loop {
		let selected = Select::new()
			.with_prompt("\nWhat would you like to do?")
			.items(&actions)
			.default(last_selected)
			.interact()
			.unwrap();
        last_selected = selected;
        let selected_action = &actions[selected];
        selected_action.run_action();
    }
}

fn show_status() {
    utils::run_command(
        "git", 
        &["status"], 
        "git status failed",
        None,

    );
}

fn show_log() {
    utils::run_command(
        "git", 
        &["--no-pager", "log", "--reverse"], 
        "git log failed",
        None,

    );
}

fn show_diff() {
    utils::run_command(
        "git", 
        &["--no-pager", "diff", "--reverse"], 
        "git diff failed",
        None,
    );
}

fn staging_mode(repo_root_dir: &str) {
    let modified_files = getters::get_modified_files();
    if modified_files.is_empty() { 
        println!("There are no modified files");
    } else {
        let modified_files_zipped: Vec<(&ProjectFile, bool)> = modified_files.iter()
            .map(|pf| {
                let can_be_staged = (&pf).is_staged();
                (pf, can_be_staged)
            })
            .collect();
        let selected = MultiSelect::new()
            .with_prompt("Which files should be staged?")
            .items_checked(&modified_files_zipped[..])
            .interact();
        if let Ok(selections) = selected {
            apply_staging(repo_root_dir, modified_files, selections);
        }
    }
}

fn apply_staging(repo_root_dir: &str, modified_files: Vec<ProjectFile>, selections: Vec<usize>) {
    let mut selection_iter = selections.into_iter();
    let mut next_selected = selection_iter.next();
    for i in 0..(&modified_files).len() {
        let current = &modified_files[i];
        let current_path = &current.file_path;
        let current_should_be_staged = match next_selected {
            None                        => false, 
            Some(next_selected_index)   => if i == next_selected_index { next_selected = selection_iter.next(); true } else { false }
        };
        if current_should_be_staged && current.is_not_staged() {
            utils::run_command_silent(
                "git", 
                &["add", current_path], 
                &format!("git add \"{}\" failed", current_path), 
                Some(&repo_root_dir),
            );
            println!("{} added to staging area", current_path);
        } else if !current_should_be_staged && current.is_staged() {
            utils::run_command_silent(
                "git", 
                &["reset", current_path], 
                &format!("git reset \"{}\" failed", current_path), 
                Some(&repo_root_dir),
            );
            println!("{} removed from staging area", current_path);
        }
    }
}

fn do_commit() {
    let staged_files = getters::get_staged_files();
    let unstaged_files = getters::get_not_staged_files();
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
    utils::run_command(
        "git", 
        &["commit"], 
        "git commit failed",
        None,
    );
}

fn do_push() {
    utils::run_command(
        "git", 
        &["push"], 
        "git push failed",
        None,
    );
}

fn do_pull() {
    utils::run_command(
        "git", 
        &["pull"], 
        "git pull failed",
        None,
    );
}

fn checkout_mode() {
    let all_commits = getters::get_commit_history();
    let selected_opt = Select::new()
        .with_prompt("Which commit would you like to checkout?")
        .items(&all_commits)
        .default(0)
        .interact_opt()
        .unwrap();
    if let Some(selected) = selected_opt {
        let selected_commit = &all_commits[selected];
        utils::run_command(
            "git", 
            &["checkout", &selected_commit.hash], 
            &format!("git checkout {} failed", selected_commit.hash),
            None,
        );
    }
}

fn branch_mode() {
    let all_branches = getters::get_branches();
    let selected_opt = Select::new()
        .with_prompt("Which branch would you like to checkout?")
        .items(&all_branches)
        .default(0)
        .interact_opt()
        .unwrap();
    if let Some(selected) = selected_opt {
        let selected_branch = &all_branches[selected];
        let mut selected_branch_name = selected_branch.name.to_owned();
        if selected_branch == &Branch::new_branch_placeholder() {
            let new_branch_name: String = Input::new()
                .with_prompt("Name of new branch")
                .interact_text()
                .unwrap();
            utils::run_command(
                "git", 
                &["branch", &new_branch_name], 
                &format!("git checkout {} failed", new_branch_name),
                None,
            );
            selected_branch_name = new_branch_name;
        }
        if !selected_branch.is_checked_out { 
            utils::run_command(
                "git", 
                &["checkout", &selected_branch_name], 
                &format!("git branch {} failed", selected_branch_name),
                None,
            );
        }
    }
}

