use libc::setuid;
use snafu::prelude::*;
use snafu::Whatever;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::prelude::CommandExt;
use std::process::Command;

fn how_to_use() {
    eprintln!("Usage: runlike <uid> <pid> <command> [<arg1> <arg2> ...]");
}
#[snafu::report]
fn main() -> Result<(), Whatever> {
    let args_string: Vec<String> = env::args().collect();
    let args_str: Vec<&str> = args_string.iter().map(AsRef::as_ref).collect();
    match &args_str[..] {
        [_self, "--help"] | [_self, "-h"] => {
            how_to_use();
        }
        [_self, uid_str, pid_str, child_program, child_args @ ..] => {
            let uid = uid_str
                .parse::<u32>()
                .with_whatever_context(|_| format!("Invalid UID: {}", uid_str))?;

            let pid = pid_str
                .parse::<i32>()
                .with_whatever_context(|_| format!("Invalid PID: {}", pid_str))?;

            let env_path = format!("/proc/{}/environ", pid);
            let env_vars = fs::read_to_string(&env_path)
                .with_whatever_context(|_| format!("Failed to read from file: {}", env_path))?;

            for env_var in env_vars.split('\0') {
                if let Some(equal_index) = env_var.find('=') {
                    let (key, value) = env_var.split_at(equal_index);
                    env::set_var(key, &value[1..]);
                }
            }

            let child_args: Vec<OsString> = child_args.iter().map(|arg| arg.into()).collect();

            unsafe {
                if setuid(uid) != 0 {
                    whatever!("Failed to call setuid.");
                }
            }
            Command::new(child_program).args(&child_args).exec();
        }
        _ => {
            how_to_use();
            whatever!("Wrong number of args. Provided: {}", args_str.len() - 1);
        }
    }
    Ok(())
}
