use regex::Regex;
use std::{
    fs::File,
    io::{Read, Write},
};

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=.env");

    let mut dotenv_contents = String::new();
    File::open(".env")?.read_to_string(&mut dotenv_contents)?;

    File::create("example.env")?.write_all(
        Regex::new(r#"".*" # Private"#)?
            .replace_all(&dotenv_contents, "")
            .as_bytes(),
    )?;

    Ok(())
}
