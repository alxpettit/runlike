use nix::unistd::Pid;
use snafu::prelude::*;
use snafu::Whatever;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::prelude::CommandExt;
use std::process::{exit, Command};

fn main() -> Result<(), Whatever> {
    match env::args().collect::<Vec<String>>().as_slice() {
        [_, pid_str, child_program, child_args @ ..] => {
            let pid = pid_str
                .parse::<i32>()
                .with_whatever_context(|_| format!("Invalid PID: {}", pid_str))?;

            let env_path = format!("/proc/{}/environ", pid);
            let env_vars = fs::read_to_string(&env_path)
                .with_whatever_context(|_| format!("Failed to read from file"))?;

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
    Ok(())
}
