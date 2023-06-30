use anyhow::Result;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> Result<()> {
    let url =
        "https://raw.githubusercontent.com/beefsack/bgg-ranking-historicals/master/2023-06-29.csv";
    let body = reqwest::blocking::get(url)?.text()?;

    let csf_file = File::create("2023-06-29.csv")?;
    let mut writer = BufWriter::new(csf_file);
    writer.write(body.as_bytes())?;

    Ok(())
}
