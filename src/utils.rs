use std::process::{
    Command,
    exit, 
};

pub fn run_command(command_name: &str, command_args: &[&str], error_message: &str, directory: Option<&str>) {
    let mut command_handle = Command::new(command_name)
        .args(command_args)
        .current_dir(directory.unwrap_or("."))
        .spawn()
        .expect(error_message);
    command_handle.wait().expect(error_message);
}

pub fn run_command_capture_output(command_name: &str, command_args: &[&str], error_message: &str, directory: Option<&str>) -> String {
    let command_output = Command::new(command_name)
        .args(command_args)
        .current_dir(directory.unwrap_or("."))
        .output()
        .expect(error_message);
    String::from_utf8_lossy(&command_output.stdout).to_string()
}

pub fn run_command_silent(command_name: &str, command_args: &[&str], error_message: &str, directory: Option<&str>) {
    Command::new(command_name)
        .args(command_args)
        .current_dir(directory.unwrap_or("."))
        .output()
        .expect(error_message);
}

pub fn exit_gracefully() {
    exit(0);
}

