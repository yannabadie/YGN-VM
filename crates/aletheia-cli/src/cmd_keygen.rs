use aletheia_core::signing::generate_keypair;
use std::fs;
use std::path::PathBuf;

pub fn run(output: Option<String>, name: String) -> Result<(), Box<dyn std::error::Error>> {
    let dir = output
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::paths::keys_dir());

    fs::create_dir_all(&dir)?;

    let (sk, vk) = generate_keypair();

    let sk_path = dir.join(format!("{name}.sec"));
    let vk_path = dir.join(format!("{name}.pub"));

    fs::write(&sk_path, hex::encode(sk))?;
    fs::write(&vk_path, hex::encode(vk))?;

    eprintln!("Keypair generated:");
    eprintln!("  Secret: {}", sk_path.display());
    eprintln!("  Public: {}", vk_path.display());

    Ok(())
}
