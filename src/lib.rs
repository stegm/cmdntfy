use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use std::ops::Deref;
use std::process::{Command as PCommand, Stdio};

pub struct NtfyConfig {
    pub url: String,
    pub token: Option<String>,
    pub cmd_args: Vec<String>,
}

pub fn run(ntfy_config: &NtfyConfig) -> Result<()> {
    let process = match PCommand::new(&ntfy_config.cmd_args[0])
        .args(&ntfy_config.cmd_args[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn process {}", &ntfy_config.cmd_args[0]))
    {
        Ok(o) => o,
        Err(e) => {
            _ = notify(
                ntfy_config,
                &format!("Failed to spawn process {}", &ntfy_config.cmd_args[0]),
                format!("Error:\n {:?}", e),
                false,
            );
            return Err(e);
        }
    };

    let output = match process.wait_with_output().context("context") {
        Ok(o) => o,
        Err(e) => {
            _ = notify(
                ntfy_config,
                &format!("Failed to wait on {}", &ntfy_config.cmd_args[0]),
                format!("Error:\n {:?}", e),
                false,
            );

            return Err(e);
        }
    };

    let stdout_data = String::from_utf8_lossy(&output.stdout);
    let stderr_data = String::from_utf8_lossy(&output.stderr);
    let return_code = output.status.code().unwrap_or(-1);

    send_message(
        ntfy_config,
        stdout_data.deref(),
        stderr_data.deref(),
        return_code,
    )?;

    Ok(())
}

fn send_message(
    ntfy_config: &NtfyConfig,
    stdout: &str,
    stderr: &str,
    exit_code: i32,
) -> Result<()> {
    let content = format!(
        "stdout:\n\n```\n{}\n```\n\nstderr:\n\n```\n{}\n```",
        stdout, stderr
    );

    notify(
        ntfy_config,
        &format!("Executing command {}", ntfy_config.cmd_args[0]),
        content,
        exit_code == 0,
    )?;

    Ok(())
}

fn notify(ntfy_config: &NtfyConfig, title: &str, content: String, success: bool) -> Result<()> {
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
