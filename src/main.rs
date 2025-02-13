use anyhow::{Context, Result};
use clap::{Arg, Command};

use cmdntfy::{run, NtfyConfig};

fn parse_args() -> Result<NtfyConfig> {
    let mut command = Command::new("cmdntfy")
        .about("Capture output of commands and send it using ntfy")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .help("ntfy URL, including topic (or NTFY_URL environment variable)"),
        )
        .arg(
            Arg::new("token")
                .short('t')
                .long("token")
                .help("ntfy token if necessary (or NTFY_TOKEN environment variable)"),
        )
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

            anyhow::bail!("no url provided (either via environment variable or argument)");
        }
    };

    let token_env = std::env::var("NTFY_TOKEN").ok();
    let token = matches
        .get_one::<String>("token")
        .map(|x| x.to_string())
        .or(token_env);

    Ok(NtfyConfig {
        url,
        token,
        cmd_args,
    })
}

fn main() -> Result<()> {
    let ntfy_config = parse_args()?;

    run(&ntfy_config).context("failed to notify command result")?;

    Ok(())
}
