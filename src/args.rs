use clap::Parser;

/// Simple cli command to show logs in a friendly way
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Finds the substring
    #[arg(short = 'c')]
    #[arg(long = "contains")]
    pub containers: Vec<String>,

    /// Defines render speed in milliseconds
    #[arg(short, long, default_value_t = 100, value_parser = render_in_range)]
    pub render: u64,
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
