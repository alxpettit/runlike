use clap::Parser;
use snafu::prelude::*;
use snafu::Whatever;
use std::env;
use std::os::unix::prelude::CommandExt;
use std::process::Command;
use sysinfo::{Pid, ProcessExt, System, SystemExt};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    pid: Pid,
    #[clap(required = true)]
    child_exe: String,
    #[clap(required = false)]
    child_args: Vec<String>,
}

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let args = Args::parse();
    let sys = System::new_all();

    let process = sys
        .process(args.pid)
        .with_whatever_context(|| format!("Could not find process with ID: {}", args.pid))?;

    let uid = process
        .user_id()
        .with_whatever_context(|| format!("Unable to retrieve UID of process: {:?}", process))?;

    let gid = process
        .group_id()
        .with_whatever_context(|| format!("Unable to retrieve GID of process: {:?}", process))?;

    for env_var in process.environ() {
        if let Some(equal_index) = env_var.find('=') {
            let (key, value) = env_var.split_at(equal_index);
            env::set_var(key, &value[1..]);
        }
    }
    let uid_out = unsafe { libc::setuid(**uid) };
    if uid_out != 0 {
        eprintln!(
            "WARNING: Failed to setuid for UID: {:?}: setuid() = {}",
            uid, uid_out
        );
    }
    let gid_out = unsafe { libc::setgid(*gid) };
    if gid_out != 0 {
        eprintln!(
            "WARNING: Failed to setgid for GID: {:?} setgid() = {}",
            gid, gid_out
        );
    }

    Command::new(args.child_exe).args(args.child_args).exec();

    Ok(())
}
