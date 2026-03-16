use aletheia_core::chain::{HashChain, Receipt};
use aletheia_core::pack::EvidencePack;
use std::fs;
use std::io::BufRead;

pub fn run(
    session: String,
    key: Option<String>,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if session.is_empty() {
        return Err("session name must not be empty".into());
    }

    let session_dir = crate::paths::session_dir(&session);
    let receipts_path = session_dir.join("receipts.jsonl");

    if !receipts_path.exists() {
        return Err(format!(
            "receipts file not found: {} — run 'capture' first",
            receipts_path.display()
        )
        .into());
    }

    // Read and re-append every receipt's event into a fresh HashChain.
    let file = fs::File::open(&receipts_path)?;
    let reader = std::io::BufReader::new(file);
    let mut chain = HashChain::new();

    for (lineno, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let receipt: Receipt = serde_json::from_str(trimmed)
            .map_err(|e| format!("invalid receipt on line {}: {}", lineno + 1, e))?;
        chain
            .append(receipt.event)
            .map_err(|e| format!("hash chain error on line {}: {}", lineno + 1, e))?;
    }

    if chain.is_empty() {
        return Err("session has no events to seal".into());
    }

    // Optionally load signing key.
    let signing_key: Option<[u8; 32]> = if let Some(key_path) = key {
        let hex_str = fs::read_to_string(&key_path)
            .map_err(|e| format!("cannot read key file '{}': {}", key_path, e))?;
        let bytes =
            hex::decode(hex_str.trim()).map_err(|e| format!("key file is not valid hex: {}", e))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| "key file must be exactly 32 bytes (64 hex chars)")?;
        Some(arr)
    } else {
        None
    };

    let event_count = chain.len();
    let pack = EvidencePack::from_chain(chain, signing_key.as_ref());

    // Determine output path.
    let out_path = output.unwrap_or_else(|| format!("{}.aletheia.json", session));

    let json = serde_json::to_string_pretty(&pack)?;
    fs::write(&out_path, &json)?;

    let merkle_hex = hex::encode(pack.merkle_root);
    let sig_count = pack.signatures.len();

    eprintln!("Sealed evidence pack:");
    eprintln!("  File:        {}", out_path);
    eprintln!("  Events:      {}", event_count);
    eprintln!("  Merkle root: {}", merkle_hex);
    eprintln!("  Signatures:  {}", sig_count);

    Ok(())
}
