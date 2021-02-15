use std::process::{
    Command,
    exit, 
};

pub fn run_command(command_name: &str, command_args: &[&str], error_message: &str) {
    let mut command_handle = Command::new(command_name)
        .args(command_args)
        .spawn()
        .expect(error_message);
    command_handle.wait().expect(error_message);
}

pub fn run_command_in_dir(in_directory: &str, command_name: &str, command_args: &[&str], error_message: &str) {
    let mut command_handle = Command::new(command_name)
        .args(command_args)
        .current_dir(in_directory)
        .spawn()
        .expect(error_message);
    command_handle.wait().expect(error_message);
}

pub fn run_command_capture_output(command_name: &str, command_args: &[&str], error_message: &str) -> String {
    let command_output = Command::new(command_name)
        .args(command_args)
        .output()
        .expect(error_message);
    String::from_utf8_lossy(&command_output.stdout).to_string()
}

pub fn exit_gracefully() {
    println!("\nThanks for stopping by!"); 
    exit(0);
}
