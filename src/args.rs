use pico_args;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml;

const HELP: &str = "\
Simple cli command to show logs in a friendly way

Usage: logss [OPTIONS]

Options:
  -c <CONTAINERS>  Finds the substring (regexp)
  -r <RENDER>      Defines render speed in milliseconds [default: 100]
  -f <FILE>        Input config file (overrides cli arguments)
  -h               Print help
";

#[derive(Debug, Serialize, Deserialize)]
pub struct Args {
    pub containers: Vec<String>,
    pub render: Option<u64>,
    pub config_file: Option<std::path::PathBuf>,
}

pub fn parse_args() -> Args {
    match parser() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    }
}

fn parser() -> Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let mut args = Args {
        containers: pargs.values_from_str("-c")?,
        config_file: pargs.opt_value_from_os_str("-f", parse_path)?,
        render: pargs
            .opt_value_from_fn("-r", render_in_range)?
            .unwrap_or(Some(100)),
    };

    let render = args.render;

    if !validate_regex(&args.containers) {
        std::process::exit(1);
    }

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    if let Some(config_file) = args.config_file {
        args = parse_yaml(config_file);
    }

    if args.render.is_none() {
        args.render = render;
    }

    Ok(args)
}

fn parse_yaml(config_file: std::path::PathBuf) -> Args {
    let f = std::fs::File::open(&config_file)
        .map_err(|_| {
            eprintln!(
                "Failed to open {file}",
                file = &config_file.as_path().display()
            );
            std::process::exit(1);
        })
        .unwrap();
    let scrape_config: Args = serde_yaml::from_reader(f)
        .map_err(|err| {
            eprintln!("Failed to parse yaml: {err}");
            std::process::exit(1);
        })
        .unwrap();
    if let Some(render) = scrape_config.render {
        if render < 25 {
            eprintln!("Values lower than 25 for 'render' make the application unresponsive.");
            std::process::exit(1);
        }
    }
    scrape_config
}

fn validate_regex(containers: &Vec<String>) -> bool {
    for c in containers {
        if Regex::new(c).is_err() {
            eprintln!("Error: Failed to parse regexp '{c}'.");
            return false;
        }
    }
    true
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

fn render_in_range(s: &str) -> Result<Option<u64>, String> {
    let render: u64 = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a valid number"))?;

    if render < 25 {
        Err("Values lower than 25 make the application unresponsive.".to_string())
    } else {
        Ok(Some(render))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_render_in_range() {
        assert_eq!(
            render_in_range("12"),
            Err("Values lower than 25 make the application unresponsive.".to_string())
        );
        assert_eq!(render_in_range("30"), Ok(Some(30)));
    }

    #[test]
    fn test_validate_regex() {
        let c = vec!["a".to_string()];
        assert_eq!(validate_regex(&c), true);

        let c = vec!["*".to_string()];
        assert_eq!(validate_regex(&c), false);
    }
}
