use aletheia_core::pack::EvidencePack;
use aletheia_core::verify::verify_pack;
use std::fs;

pub fn run(pack: String, key: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let json_str = fs::read_to_string(&pack)
        .map_err(|e| format!("cannot read pack file '{}': {}", pack, e))?;

    let evidence: EvidencePack = serde_json::from_str(&json_str)
        .map_err(|e| format!("invalid evidence pack: {}", e))?;

    // Optionally load verifying key.
    let verifying_key: Option<[u8; 32]> = if let Some(key_path) = key {
        let hex_str = fs::read_to_string(&key_path)
            .map_err(|e| format!("cannot read key file '{}': {}", key_path, e))?;
        let bytes = hex::decode(hex_str.trim())
            .map_err(|e| format!("key file is not valid hex: {}", e))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| "key file must be exactly 32 bytes (64 hex chars)")?;
        Some(arr)
    } else {
        None
    };

    match verify_pack(&evidence, verifying_key.as_ref()) {
        Ok(result) => {
            let output = serde_json::json!({
                "status": "verified",
                "receipt_count": result.receipt_count,
                "chain_ok": result.chain_ok,
                "merkle_ok": result.merkle_ok,
                "chain_head_ok": result.chain_head_ok,
                "signatures_valid": result.signatures_valid,
                "signatures_total": result.signatures_total,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        Err(e) => {
            let output = serde_json::json!({
                "status": "failed",
                "error": e.to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            std::process::exit(1);
        }
    }

    Ok(())
}
