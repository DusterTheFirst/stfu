use regex::Regex;
use std::{env, fs};

fn main() -> anyhow::Result<()> {
    if fs::metadata(".env").is_ok() && env::var("CI").map(|v| v == "true").unwrap_or_default() {
        println!("cargo:rerun-if-changed=.env");

        let contents = fs::read_to_string(".env")?;

        fs::write(
            "example.env",
            Regex::new(r#"".*" # Private"#)?
                .replace_all(&contents, "")
                .as_bytes(),
        )?
    }

    Ok(())
}
