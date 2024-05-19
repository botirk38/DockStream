use anyhow::{Context, Result};
use std::fs::copy;
use std::io::Write;
use std::os::unix::fs;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.

    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    std::fs::create_dir_all("/sandbox/usr/local/bin").expect("Failed to create directory");
    copy(
        "/usr/local/bin/docker-explorer",
        "/sandbox/usr/local/bin/docker-explorer",
    )
    .expect("Failed to copy file");
    fs::chroot("/sandbox").expect("unable to chroot.");
    std::env::set_current_dir("/")?;
    std::fs::create_dir_all("dev/null")?;

    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;

    if output.status.success() {
        std::process::exit(0);
    } else {
        std::process::exit(output.status.code().unwrap_or(1));
    }

}

