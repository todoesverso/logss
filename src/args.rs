use pico_args;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml;

const HELP: &str = "\
Simple cli command to show logs in a friendly way

Usage: logss [OPTIONS]

Options:
  -c <CONTAINERS>  Finds the substring (regexp)
  -C <COMMAND>     Gets input from this command
  -r <RENDER>      Defines render speed in milliseconds [default: 100]
  -f <FILE>        Input config file (overrides cli arguments)
  -V               Start in vertical view mode
  -h               Print help
";

#[derive(Debug, Serialize, Deserialize)]
pub struct Args {
    pub containers: Vec<String>,
    pub vertical: Option<bool>,
    pub render: Option<u64>,
    pub command: Option<Vec<String>>,
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

fn parser() -> Result<Args, Box<dyn std::error::Error>> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let mut args = Args {
        containers: pargs.values_from_str("-c")?,
        command: pargs.opt_value_from_fn("-C", parse_cmd)?,
        config_file: pargs.opt_value_from_os_str("-f", parse_path)?,
        vertical: pargs.contains("-V").then_some(true),
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
        args = parse_yaml(config_file)?;
    }

    if args.render.is_none() {
        args.render = render;
    }

    Ok(args)
}

fn parse_yaml(config_file: std::path::PathBuf) -> Result<Args, Box<dyn std::error::Error>> {
    let f = std::fs::File::open(config_file)?;
    let scrape_config: Args = serde_yaml::from_reader(f)?;
    if let Some(render) = scrape_config.render {
        if render < 25 {
            return Err("Values lower than 25 make the application unresponsive.".into());
        }
    }
    Ok(scrape_config)
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

fn parse_cmd(s: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(s.split_whitespace().map(str::to_string).collect())
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

    #[test]
    fn test_parse_yaml() {
        let path = std::path::PathBuf::from("example_config.yml");
        let args = parse_yaml(path).unwrap();
        assert_eq!(args.render.unwrap(), 26);
        assert_eq!(
            args.containers,
            vec![
                "to".to_string(),
                "be".to_string(),
                "or".to_string(),
                "not".to_string(),
                "to.*be".to_string()
            ]
        );
    }
}
