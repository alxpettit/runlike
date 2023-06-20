// TODO: add linux capabilities

use clap::Parser;
use snafu::prelude::*;
use snafu::ResultExt;
use snafu::Whatever;
use std::env;
use std::os::unix::prelude::CommandExt;
use std::process::Command;
use sysinfo::{Pid, Process, ProcessExt, System, SystemExt, User, UserExt};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    pid: Option<Pid>,
    #[clap(required = true)]
    child_exe: String,
    #[clap(required = false)]
    child_args: Vec<String>,
}

const DETECTED_ENTRYPOINTS: &'static [&'static str] = &[".startplasma-wa", ".startplasma-x1"];
fn detect_process<'a>(sys: &'a System) -> Result<&'a Process, Whatever> {
    for (_, process) in sys.processes() {
        if DETECTED_ENTRYPOINTS.contains(&process.name()) {
            return Ok(process);
        }
    }

    whatever!(
        "Could not automatically detect process with name() matching: {:?}",
        DETECTED_ENTRYPOINTS
    )
}

fn get_user_from_proc<'a>(sys: &'a System, process: &Process) -> Result<&'a User, Whatever> {
    let uid = process
        .effective_user_id()
        .with_whatever_context(|| format!("Unable to retrieve UID of process: {:?}", process))?;

    let user = sys
        .get_user_by_id(uid)
        .with_whatever_context(|| format!("Could not resolve user name from ID: {:?}", uid))?;
    Ok(user)
}

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let args = Args::parse();
    let sys = System::new_all();

    let process = match args.pid {
        Some(pid) => sys
            .process(pid)
            .with_whatever_context(|| format!("Could not find process with ID: {:?}", args.pid))?,
        None => detect_process(&sys)
            .with_whatever_context(|_| format!("Can't detect process to use."))?,
    };

    let user = get_user_from_proc(&sys, &process)?;
    let name = user.name();
    let groups = user.groups();
    let environ = process.environ();

    privdrop::PrivDrop::default()
        .user(name)
        .group_list(groups)
        .apply()
        .with_whatever_context(|_| format!("Could not set privs to match target user: {}", name))?;

    for env_var in environ {
        if let Some(equal_index) = env_var.find('=') {
            let (key, value) = env_var.split_at(equal_index);
            env::set_var(key, &value[1..]);
        }
    }

    Command::new(args.child_exe).args(args.child_args).exec();

    Ok(())
}
