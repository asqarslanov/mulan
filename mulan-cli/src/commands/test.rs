use anyhow::Result;

pub fn main() -> Result<()> {
    let translation = mulan_parser::parse_translation("test.json5")?;
    println!("{translation:?}");
    Ok(())
}
