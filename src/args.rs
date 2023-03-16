use pico_args;

const HELP: &str = "\
Simple cli command to show logs in a friendly way

Usage: logss [OPTIONS]

Options:
  -c <CONTAINERS>  Finds the substring
  -r <RENDER>      Defines render speed in milliseconds [default: 100]
  -h               Print help
";

#[derive(Debug)]
pub struct Args {
    pub containers: Vec<String>,
    pub render: u64,
}

pub fn parse_args() -> Args {
    let args = match parser() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };
    args
}

fn parser() -> Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = Args {
        containers: pargs.values_from_str("-c")?,
        render: pargs
            .opt_value_from_fn("-r", render_in_range)?
            .unwrap_or(100),
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

fn render_in_range(s: &str) -> Result<u64, String> {
    let render: u64 = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a valid number"))?;

    if render < 25 {
        Err("Values lower than 25 make the application unresponsive.".to_string())
    } else {
        Ok(render)
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
        assert_eq!(render_in_range("30"), Ok(30));
    }
}
