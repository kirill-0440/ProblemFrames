use anyhow::{anyhow, Result};
use pf_sysml_api::{run_smoke, SmokeConfig};

fn usage() -> &'static str {
    "Usage: pf_sysml_api smoke [--endpoint=<url>] [--dry-run]"
}

fn parse_args(args: &[String]) -> Result<SmokeConfig> {
    if args.len() < 2 || args[1] != "smoke" {
        return Err(anyhow!(usage()));
    }

    let mut endpoint = None;
    let mut dry_run = false;
    let mut index = 2;

    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--endpoint=") {
            endpoint = Some(value.to_string());
            index += 1;
            continue;
        }
        if arg == "--endpoint" {
            if index + 1 >= args.len() {
                return Err(anyhow!("missing value for --endpoint"));
            }
            endpoint = Some(args[index + 1].clone());
            index += 2;
            continue;
        }
        if arg == "--dry-run" {
            dry_run = true;
            index += 1;
            continue;
        }
        return Err(anyhow!("unknown option '{}'. {}", arg, usage()));
    }

    Ok(SmokeConfig { endpoint, dry_run })
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let config = parse_args(&args)?;
    let verdict = run_smoke(&config);
    println!("{}", serde_json::to_string_pretty(&verdict)?);
    Ok(())
}
