use std::{
    fs::{remove_file, OpenOptions},
    str::FromStr,
};

use pico_args;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml;

const HELP: &str = "\
Simple CLI command to display logs in a user-friendly way

Usage: logss [OPTIONS]

Options:
  -c <CONTAINERS>  Specify substrings (regex patterns)
  -e               Exit on empty input [default: false]
  -s               Start in single view mode [default: false]
  -C <COMMAND>     Get input from a command
  -f <FILE>        Input configuration file (overrides CLI arguments)
  -o <OUTPUT_PATH> Specify the output path for matched patterns
  -r <RENDER>      Define render speed in milliseconds [default: 100]
  -t <THREADS>     Number of threads per container for triggers [default: 1]
  -V               Start in vertical view mode
  -h               Print help
";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LocalContainer {
    pub re: String,
    pub trigger: Option<String>,
    pub timeout: Option<u64>,
}

impl FromStr for LocalContainer {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split(',').collect();

        if parts.len() > 3 {
            return Err("Expected not more than 2 comma-separated parts");
        }

        let re = parts[0].trim().to_string();
        let trigger = if parts.len() < 2 || parts[1].trim().is_empty() {
            None
        } else {
            Some(parts[1].trim().to_string())
        };
        let timeout = if parts.len() < 3 || parts[2].trim().is_empty() {
            Some(1)
        } else {
            let timeout: u64 = parts[2].trim().parse().unwrap_or(1);
            Some(timeout)
        };

        Ok(LocalContainer {
            re,
            trigger,
            timeout,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Args {
    pub containers: Vec<LocalContainer>,
    pub exit: Option<bool>,
    pub vertical: Option<bool>,
    pub single: Option<bool>,
    pub render: Option<u64>,
    pub threads: Option<u64>,
    pub command: Option<Vec<String>>,
    pub output: Option<std::path::PathBuf>,
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
        output: pargs.opt_value_from_os_str("-o", validate_path)?,
        exit: pargs.contains("-e").then_some(true),
        single: pargs.contains("-s").then_some(true),
        vertical: pargs.contains("-V").then_some(true),
        render: pargs
            .opt_value_from_fn("-r", render_in_range)?
            .unwrap_or(Some(100)),
        threads: pargs
            .opt_value_from_fn("-t", render_in_range)?
            .unwrap_or(Some(4)),
    };

    let render = args.render;

    if !validate_regex(&args.containers) {
        std::process::exit(1);
    }

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Error: non valid arguments: {:?}.", remaining);
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
    Ok(scrape_config)
}

fn validate_regex(containers: &Vec<LocalContainer>) -> bool {
    for c in containers {
        if Regex::new(&c.re).is_err() {
            eprintln!("Error: Failed to parse regexp '{c:?}'.");
            return false;
        }
    }
    true
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

fn validate_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, String> {
    let path: std::path::PathBuf = s.into();
    if !path.is_dir() {
        return Err(format!("{} is not a valid path", path.display()));
    }
    /*  TODO: re write once you learn some real rust
     * Not proud of this but is the simplest way I found to test
     * write permissions in a directory
     */
    let test_file_name = format!("{}/.logss", path.to_string_lossy());

    let a = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&test_file_name);

    match a {
        Ok(_) => {
            remove_file(test_file_name).expect("Failed to delete sentinel file");
            Ok(path)
        }
        Err(error) => Err(error.to_string()),
    }
}

fn parse_cmd(s: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(s.split_whitespace().map(str::to_string).collect())
}

fn render_in_range(s: &str) -> Result<Option<u64>, String> {
    let render: u64 = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a valid number"))?;

    Ok(Some(render))
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_os = "windows"))]
    use std::os::unix::fs::DirBuilderExt;
    use std::{
        ffi::OsStr,
        fs::{remove_dir_all, DirBuilder},
        path::PathBuf,
    };

    use super::*;
    #[test]
    fn test_render_in_range() {
        assert_eq!(render_in_range("30"), Ok(Some(30)));
    }

    #[test]
    fn test_validate_regex() {
        let c = vec![LocalContainer {
            re: "a".to_string(),
            trigger: None,
            timeout: None,
        }];
        assert!(validate_regex(&c));

        let c = vec![LocalContainer {
            re: "*".to_string(),
            trigger: None,
            timeout: None,
        }];
        assert!(!validate_regex(&c));
    }

    #[test]
    fn test_validate_path_non_valid() {
        let resp = Err("non_valid_path is not a valid path".to_string());
        assert_eq!(validate_path(OsStr::new("non_valid_path")), resp);
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_validate_path_no_perm() {
        let _ = remove_dir_all("test-sarasa");
        let mut dir = DirBuilder::new();
        dir.mode(0o444);
        dir.recursive(true).create("test-sarasa").unwrap();
        let resp = Err("Permission denied (os error 13)".to_string());
        assert_eq!(validate_path(OsStr::new("test-sarasa")), resp);
        let _ = remove_dir_all("test-sarasa");
    }

    #[test]
    fn test_validate_path() {
        let _ = remove_dir_all("test-sarasa");
        let path = PathBuf::from("test-sarasa");
        let mut dir = DirBuilder::new();
        dir.recursive(true).create("test-sarasa").unwrap();
        assert_eq!(validate_path(OsStr::new("test-sarasa")), Ok(path));
        let _ = remove_dir_all("test-sarasa");
    }

    #[test]
    fn test_parse_yaml() {
        let path = std::path::PathBuf::from("example_config.yml");
        let args = parse_yaml(path).unwrap();
        assert_eq!(args.render.unwrap(), 26);
        assert_eq!(
            args.containers,
            vec![
                LocalContainer {
                    re: "to".to_string(),
                    trigger: Some("echo $(date) >> /tmp/dates.txt".to_string()),
                    timeout: Some(1),
                },
                LocalContainer {
                    re: "be".to_string(),
                    trigger: Some("echo __line__ >> /tmp/match_lines.txt".to_string()),
                    timeout: Some(1),
                },
                LocalContainer {
                    re: "or".to_string(),
                    trigger: None,
                    timeout: Some(1),
                },
                LocalContainer {
                    re: "not".to_string(),
                    trigger: None,
                    timeout: Some(1),
                },
                LocalContainer {
                    re: "to.*be".to_string(),
                    trigger: None,
                    timeout: Some(1),
                },
            ]
        );
    }
}
