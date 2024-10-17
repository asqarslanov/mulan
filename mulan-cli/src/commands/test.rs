pub fn main() {
    let config = mulan_config::parse().unwrap();
    let translation = mulan_parser::parse_translation("en-US", &config).unwrap();
    println!("{translation:?}");
}
