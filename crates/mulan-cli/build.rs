use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=../../mulan.toml");
    println!("cargo::rerun-if-changed=../../locales/");
    _ = Command::new("mulan").arg("gen").spawn();
}
