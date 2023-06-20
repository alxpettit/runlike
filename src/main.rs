// TODO: add linux capabilities

use clap::Parser;
use snafu::prelude::*;
use snafu::Whatever;
use std::env;
use std::os::unix::prelude::CommandExt;
use std::process::Command;
use sysinfo::{Pid, ProcessExt, System, SystemExt, UserExt};

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
        .effective_user_id()
        .with_whatever_context(|| format!("Unable to retrieve UID of process: {:?}", process))?;

    let user = sys
        .get_user_by_id(uid)
        .with_whatever_context(|| format!("Could not resolve user name from ID: {:?}", uid))?;

    privdrop::PrivDrop::default()
        .user(user.name())
        .group_list(user.groups())
        .apply()
        .with_whatever_context(|_| {
            format!("Could not set privs to match target user: {}", user.name())
        })?;

    for env_var in process.environ() {
        if let Some(equal_index) = env_var.find('=') {
            let (key, value) = env_var.split_at(equal_index);
            env::set_var(key, &value[1..]);
        }
    }

    Command::new(args.child_exe).args(args.child_args).exec();

    Ok(())
}
