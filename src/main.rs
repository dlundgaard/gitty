use gitty::modes;
use clap::Command;

fn main() {
    Command::new(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Interactive command line interface for enhanced workflow when using the Git CLI")
        .get_matches();
    modes::select_command_mode();
}

