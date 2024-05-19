use anyhow::{Context, Result};
use std::env;
use std::fs::{self, copy};
use std::os::unix::fs as unix_fs;
use std::process::{Command, exit};
use std::io::Write;

fn main() -> Result<()> {
    setup_sandbox_environment()?;

    let args: Vec<_> = env::args().collect();
    if args.len() < 5 {
        anyhow::bail!("Not enough arguments provided.");
    }
    let command = &args[3];
    let command_args = &args[4..];

    execute_command(command, command_args)
}

fn setup_sandbox_environment() -> Result<()> {
    fs::create_dir_all("/sandbox/usr/local/bin")?;
    copy(
        "/usr/local/bin/docker-explorer",
        "/sandbox/usr/local/bin/docker-explorer",
    )?;
    unix_fs::chroot("/sandbox")?;
    env::set_current_dir("/")?;
    fs::create_dir_all("dev/null")?;

    Ok(())
}

fn execute_command(command: &str, command_args: &[String]) -> Result<()> {
    let output = Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| format!("Tried to run '{}' with arguments {:?}", command, command_args))?;

    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;

    if output.status.success() {
        exit(0);
    } else {
        exit(output.status.code().unwrap_or(1));
    }
}

