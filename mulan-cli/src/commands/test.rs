use anyhow::Result;

pub fn main() -> Result<()> {
    let config = mulan_config::parse()?;
    println!("{config:?}");
    // let translation = mulan_parser::parse_translation("en-US")?;
    // println!("{translation:?}");
    Ok(())
}
