use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let out_dir: PathBuf = env::var_os("OUT_DIR").context("OUT_DIR is not set")?.into();

    println!("cargo::rerun-if-changed=src/version_info.ron");
    let version_info = fs::read_to_string("src/version_info.ron")?;
    let version_info = mazecode_codegen::generate(&ron::from_str(&version_info)?);

    fs::write(
        out_dir.join("generated.rs"),
        prettyplease::unparse(&version_info),
    )?;

    Ok(())
}
