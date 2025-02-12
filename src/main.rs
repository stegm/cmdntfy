use anyhow::{Context, Result};
use clap::{Arg, Command};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use std::ops::Deref;
use std::process::{Command as PCommand, Stdio};

struct NtfyConfig {
    url: String,
    token: Option<String>,
}

fn _notify(
    ntfy_config: &NtfyConfig,
    title: &str,
    content: String,
    success: bool,
) -> Result<()> {
    let mut headers = HeaderMap::new();
    headers.insert("X-Markdown", HeaderValue::from_static("yes"));
    headers.insert("X-Title", HeaderValue::from_str(title).unwrap());
    if success {
        headers.insert("X-Priority", HeaderValue::from_static("low"));
        headers.insert("X-Tags", HeaderValue::from_static("heavy_check_mark"));
    } else {
        headers.insert("X-Priority", HeaderValue::from_static("high"));
        headers.insert("X-Tags", HeaderValue::from_static("rotating_light"));
    }

    if let Some(t) = &ntfy_config.token {
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", t)).unwrap(),
        );
    }

    let client = Client::new();
    let _response = client
        .post(&ntfy_config.url)
        .body(content)
        .headers(headers)
        .send()
        .context("failed to send message")?;

    Ok(())
}

fn _send_message(
    ntfy_config: &NtfyConfig,
    cmd: &[String],
    stdout: &str,
    stderr: &str,
    exit_code: i32,
) -> Result<()> {
    let content = format!(
        "stdout:\n\n```\n{}\n```\n\nstderr:\n\n```\n{}\n```",
        stdout, stderr
    );

    _notify(
        ntfy_config,
        &format!("Executing command {}", cmd[0]),
        content,
        exit_code == 0,
    )?;

    Ok(())
}

fn run(ntfy_config: &NtfyConfig, cmd: Vec<String>) -> Result<()> {
    let process = match PCommand::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn process {}", &cmd[0]))
    {
        Ok(o) => o,
        Err(e) => {
            _ = _notify(
                ntfy_config,
                &format!("Failed to spawn process {}", &cmd[0]),
                format!("Error:\n {:?}", e),
                false,
            );
            return Err(e);
        }
    };

    let output = match process.wait_with_output().context("context") {
        Ok(o) => o,
        Err(e) => {
            _ = _notify(
                ntfy_config,
                &format!("Failed to wait on {}", &cmd[0]),
                format!("Error:\n {:?}", e),
                false,
            );

            return Err(e);
        }
    };

    let stdout_data = String::from_utf8_lossy(&output.stdout);
    let stderr_data = String::from_utf8_lossy(&output.stderr);
    let return_code = output.status.code().unwrap_or(-1);

    _send_message(
        ntfy_config,
        &cmd,
        stdout_data.deref(),
        stderr_data.deref(),
        return_code,
    )?;

    Ok(())
}

fn main() -> Result<()> {
    let mut command = Command::new("cmdntfy")
        .about("Capture output of commands and send it using ntfy")
        .arg(Arg::new("url").short('u').long("url").help(
            "ntfy URL, including topic (or NTFY_URL environment variable)",
        ))
        .arg(Arg::new("token").short('t').long("token").help(
            "ntfy token if necessary (or NTFY_TOKEN environment variable)",
        ))
        .arg(
            Arg::new("cmd_args")
                .help("Command-line (executable and arguments)")
                .required(true)
                .num_args(1..),
        );

    let usage = command.render_usage();
    let matches = command.get_matches();

    let cmd_args = matches
        .get_many::<String>("cmd_args")
        .unwrap()
        .cloned()
        .collect();

    let url_env = std::env::var("NTFY_URL").ok();
    let url = match matches
        .get_one::<String>("url")
        .map(|x| x.to_string())
        .or(url_env)
    {
        Some(o) => o,
        None => {
            eprintln!("{usage}");

            anyhow::bail!(
                "no url provided (either via environment variable or argument)"
            );
        }
    };

    let token_env = std::env::var("NTFY_TOKEN").ok();
    let token = matches
        .get_one::<String>("token")
        .map(|x| x.to_string())
        .or(token_env);

    let ntfy_config = NtfyConfig { url, token };

    run(&ntfy_config, cmd_args).context("failed to notify command result")?;

    Ok(())
}
