use anyhow::anyhow;
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::env;
use std::fs::{self, copy};
use std::io::Write;
use std::os::unix::fs as unix_fs;
use std::process::{exit, Command};
use tar::Archive;

static DOCKER_HUB: &str = "registry.hub.docker.com";

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 5 {
        anyhow::bail!("Not enough arguments provided.");
    }
    let command = &args[3];
    let command_args = &args[4..];
    let image = &args[2];

    // Pull the image
    pull_image(image)?;

    setup_sandbox_environment()?;

    isolate_process()?;

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
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;

    if output.status.success() {
        exit(0);
    } else {
        exit(output.status.code().unwrap_or(1));
    }
}

fn isolate_process() -> Result<()> {
    unsafe { libc::unshare(libc::CLONE_NEWPID) };
    Ok(())
}

fn pull_image(image: &str) -> Result<()> {
    // Authenticate with the registry

    let auth_token = authenticate_with_registry(image)?;

    let manifest = fetch_manifest(image, &auth_token)?;

    let layer = &manifest["layers"].as_array().unwrap()[0]; // Get the first layer

    let digest = layer["digest"].as_str().unwrap(); // Get the digest of the layer

    let layer_data = fetch_layer(image, digest, &auth_token)?;

    let tar = GzDecoder::new(&layer_data[..]);

    let mut archive = Archive::new(tar);
    archive.unpack("/sandbox")?;

    Ok(())
}

fn authenticate_with_registry(image: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let auth_request = client.get(&format!(
            "https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/{}:pull",
            image
        ))
        .send()
        .with_context(|| format!("Failed to fetch auth token for image '{}'", image))?;

    let auth_response = auth_request
        .json::<serde_json::Value>()
        .with_context(|| format!("Failed to parse auth token response for image '{}'", image))?;

    let auth_token = auth_response
        .get("token")
        .ok_or_else(|| anyhow!("Token not found in response"))
        .and_then(|t| t.as_str().ok_or_else(|| anyhow!("Token is not a string")))
        .with_context(|| format!("Failed to get auth token for image '{}'", image))?;

    Ok(auth_token.to_string())
}

fn fetch_manifest(image: &str, auth_token: &str) -> Result<serde_json::Value> {
    let client = reqwest::blocking::Client::new();
    let manifest_request = client
        .get(&format!(
            "https://{}/v2/library/{}/manifests/latest",
            DOCKER_HUB, image
        ))
        .header(
            "Accept",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .with_context(|| format!("Failed to fetch manifest for image '{}'", image))?;

    let manifest = manifest_request
        .json::<serde_json::Value>()
        .with_context(|| format!("Failed to parse manifest for image '{}'", image))?;

    Ok(manifest)
}

fn fetch_layer(image: &str, digest: &str, auth_token: &str) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::new();
    let layer_request = client
        .get(&format!(
            "https://{}/v2/library/{}/blobs/{}",
            DOCKER_HUB, image, digest
        ))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .with_context(|| format!("Failed to fetch layer for image '{}'", image))?;

    let layer_data = layer_request
        .bytes()
        .with_context(|| format!("Failed to get layer data for image '{}'", image))?;

    Ok(layer_data.to_vec())
}


