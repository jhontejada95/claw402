use std::{fs, path::PathBuf};

use solana_sdk::{
    signature::{write_keypair_file, Keypair},
    signer::Signer,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(".tmp/claw402-devnet/payer.json");
    if path.exists() {
        return Err(format!("refusing to overwrite existing wallet: {}", path.display()).into());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let keypair = Keypair::new();
    write_keypair_file(&keypair, &path)?;

    println!("wallet_created=true");
    println!("network=solana-devnet");
    println!("public_address={}", keypair.pubkey());
    println!("secret_path={}", path.display());
    println!("secret_printed=false");
    Ok(())
}
