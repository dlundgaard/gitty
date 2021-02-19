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
};

pub fn select_command_mode() {
    let git_root_dir = getters::get_git_root_dir();
    let actions: Vec<Action> = vec![
        Action::new("status", || show_status()),
        Action::new("log", || show_log()),
        Action::new("diff", || show_diff()),
        Action::new("stage", || staging_mode(&git_root_dir)),
        Action::new("unstage", || unstaging_mode(&git_root_dir)),
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
        "git status failed"
    );
}

fn show_log() {
    utils::run_command(
        "git", 
        &["--no-pager", "log", "--reverse"], 
        "git log failed"
    );
}

fn show_diff() {
    utils::run_command(
        "git", 
        &["--no-pager", "diff", "--reverse"], 
        "git diff failed"
    );
}

fn staging_mode(repo_root_dir: &str) {
    let unstaged_files = getters::get_not_staged_files();
    if unstaged_files.is_empty() { 
        println!("There are no unstaged files");
    } else {
        let selections = MultiSelect::new()
            .with_prompt("Which files should be staged?")
            .items(&unstaged_files)
            .interact()
            .unwrap();
        for selected in selections {
            let file_to_be_staged = &unstaged_files[selected].file_path;
            utils::run_command_in_dir(
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
    let staged_files = getters::get_staged_files();
    if staged_files.is_empty() { 
        println!("There are no staged files");
    } else {
        let selections = MultiSelect::new()
            .with_prompt("Which files should be unstaged?")
            .items(&staged_files)
            .interact()
            .unwrap();
        for selected in selections {
            let file_to_be_unstaged = &staged_files[selected].file_path;
            utils::run_command_in_dir(
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
        "git commit failed"
    );
}

fn do_push() {
    utils::run_command(
        "git", 
        &["push"], 
        "git push failed"
    );
}

fn do_pull() {
    utils::run_command(
        "git", 
        &["pull"], 
        "git pull failed"
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
            &format!("git checkout {} failed", selected_commit.hash)
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
                &format!("git checkout {} failed", new_branch_name)
            );
            selected_branch_name = new_branch_name;
        }
        if !selected_branch.is_checked_out { 
            utils::run_command(
                "git", 
                &["checkout", &selected_branch_name], 
                &format!("git branch {} failed", selected_branch_name)
            );
        }
    }
}

