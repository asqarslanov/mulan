use anyhow::Result;

pub fn main() -> Result<()> {
    let translation = mulan_parser::parse_translation("en-US")?;
    println!("{translation:?}");
    Ok(())
}
