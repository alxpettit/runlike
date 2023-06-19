use nix::unistd::Pid;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::prelude::CommandExt;
use std::process::{exit, Command};

fn main() {
    match env::args().collect::<Vec<String>>().as_slice() {
        [_, pid_str, child_program, child_args @ ..] => {
            let pid = match pid_str.parse::<i32>() {
                Ok(value) => Pid::from_raw(value),
                Err(_) => {
                    eprintln!("Invalid PID");
                    exit(1);
                }
            };

            let env_path = format!("/proc/{}/environ", pid);
            let env_vars = match fs::read_to_string(&env_path) {
                Ok(content) => content,
                Err(_) => {
                    eprintln!("Failed to read environment variables");
                    exit(1);
                }
            };

            for env_var in env_vars.split('\0') {
                if let Some(equal_index) = env_var.find('=') {
                    let (key, value) = env_var.split_at(equal_index);
                    env::set_var(key, &value[1..]);
                }
            }

            let child_args: Vec<OsString> = child_args.iter().map(|arg| arg.into()).collect();
            Command::new(child_program).args(&child_args).exec();
        }
        _ => {
            eprintln!("Usage: <program> <pid> <command> [<arg1> <arg2> ...]");
            exit(1);
        }
    }
}
