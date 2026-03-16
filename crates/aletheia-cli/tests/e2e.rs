//! End-to-end integration tests for the Aletheia CLI.
//!
//! These tests spawn the real `aletheia` binary (via `cargo run -p aletheia-cli`)
//! and exercise the full capture → seal → verify → export pipeline.
//!
//! Session names are made unique per test-process run using the process ID to
//! avoid receipt accumulation when tests are re-run (the `capture` command
//! appends to an existing session).  A cleanup step removes the session from
//! the platform config dir at the end of each test.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Return the workspace root (two levels up from this crate's Cargo.toml).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// Build a `Command` that runs `aletheia` via `cargo run`.
fn aletheia_cmd(workspace: &PathBuf) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--quiet")
        .arg("-p")
        .arg("aletheia-cli")
        .arg("--")
        .current_dir(workspace);
    cmd
}

/// Generate a session name that is unique to this process invocation.
///
/// Combining the process ID with a per-call counter prevents two tests running
/// in the same process from sharing a session name, and prevents re-runs from
/// appending to a leftover session.
fn unique_session(base: &str) -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}-{}", base, std::process::id(), n)
}

/// Return the platform config directory where `aletheia` stores sessions.
///
/// This mirrors the logic in `crates/aletheia-cli/src/paths.rs`.
fn aletheia_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aletheia")
}

/// Delete a session directory from the platform config dir.
///
/// Called at the end of every test to keep the real user config clean.
fn cleanup_session(session: &str) {
    let dir = aletheia_config_dir().join("sessions").join(session);
    let _ = fs::remove_dir_all(dir);
}

/// Create a temporary output directory for pack files and key files.
fn make_output_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aletheia-e2e-out-{label}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create output dir");
    dir
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// Full unsigned pipeline: capture → seal → verify → export (JSON).
#[test]
fn full_pipeline_unsigned() {
    let ws = workspace_root();
    let session = unique_session("e2e-unsigned");
    let out_dir = make_output_dir(&session);

    // ── 1. Capture ────────────────────────────────────────────────────────────
    let events_jsonl = concat!(
        r#"{"kind":"ToolUse","source":"agent","payload":{"tool":"read_file","path":"src/main.rs"}}"#,
        "\n",
        r#"{"kind":"PolicyCheck","source":"agent","payload":{"policy":"no-secrets","result":"pass"}}"#,
        "\n",
        r#"{"kind":"Commit","source":"agent","payload":{"message":"initial commit","sha":"abc1234"}}"#,
        "\n",
    );

    let mut capture_cmd = aletheia_cmd(&ws);
    capture_cmd
        .args(["capture", "--session", &session])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut capture_child = capture_cmd.spawn().expect("spawn capture");
    capture_child
        .stdin
        .take()
        .expect("stdin")
        .write_all(events_jsonl.as_bytes())
        .expect("write stdin");
    let capture_out = capture_child.wait_with_output().expect("capture output");

    assert!(
        capture_out.status.success(),
        "capture failed:\nstderr: {}",
        String::from_utf8_lossy(&capture_out.stderr)
    );

    let capture_stderr = String::from_utf8_lossy(&capture_out.stderr);
    assert!(
        capture_stderr.contains("Captured") || capture_stderr.contains("3"),
        "expected event count in stderr, got: {capture_stderr}"
    );

    // ── 2. Seal ───────────────────────────────────────────────────────────────
    let pack_path = out_dir.join("test.aletheia.json");
    let pack_path_str = pack_path.to_str().expect("pack path to str");

    let mut seal_cmd = aletheia_cmd(&ws);
    seal_cmd
        .args(["seal", "--session", &session, "--output", pack_path_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let seal_out = seal_cmd.output().expect("seal output");
    assert!(
        seal_out.status.success(),
        "seal failed:\nstderr: {}",
        String::from_utf8_lossy(&seal_out.stderr)
    );
    assert!(
        pack_path.exists(),
        "pack file was not created at {pack_path_str}"
    );

    // ── 3. Verify ─────────────────────────────────────────────────────────────
    let mut verify_cmd = aletheia_cmd(&ws);
    verify_cmd
        .args(["verify", pack_path_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let verify_out = verify_cmd.output().expect("verify output");
    assert!(
        verify_out.status.success(),
        "verify exited non-zero:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&verify_out.stdout),
        String::from_utf8_lossy(&verify_out.stderr)
    );

    let verify_stdout = String::from_utf8_lossy(&verify_out.stdout);
    assert!(
        verify_stdout.contains("verified"),
        "expected 'verified' in verify output, got: {verify_stdout}"
    );

    // Parse the JSON output and check structural integrity.
    let verify_json: serde_json::Value =
        serde_json::from_str(&verify_stdout).expect("verify output is valid JSON");
    assert_eq!(
        verify_json["status"], "verified",
        "status should be verified"
    );
    assert_eq!(
        verify_json["receipt_count"], 3,
        "should have captured 3 receipts"
    );
    assert_eq!(verify_json["chain_ok"], true, "hash chain should be intact");
    assert_eq!(
        verify_json["merkle_ok"], true,
        "merkle root should be intact"
    );

    // ── 4. Export (JSON format) ───────────────────────────────────────────────
    let mut export_cmd = aletheia_cmd(&ws);
    export_cmd
        .args(["export", "--format", "json", pack_path_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let export_out = export_cmd.output().expect("export output");
    assert!(
        export_out.status.success(),
        "export failed:\nstderr: {}",
        String::from_utf8_lossy(&export_out.stderr)
    );

    let export_stdout = String::from_utf8_lossy(&export_out.stdout);
    let export_json: serde_json::Value =
        serde_json::from_str(&export_stdout).expect("export output is valid JSON");
    assert_eq!(
        export_json["version"], "1.0",
        "pack version should be '1.0'"
    );
    assert_eq!(
        export_json["receipts"].as_array().map(|a| a.len()),
        Some(3),
        "exported pack should contain 3 receipts"
    );

    // ── Cleanup ───────────────────────────────────────────────────────────────
    cleanup_session(&session);
    let _ = fs::remove_dir_all(&out_dir);
}

/// Full signed pipeline: keygen → capture → seal (with key) → verify.
#[test]
fn full_pipeline_signed() {
    let ws = workspace_root();
    let session = unique_session("e2e-signed");
    let out_dir = make_output_dir(&session);

    // ── 1. Keygen ─────────────────────────────────────────────────────────────
    let keys_dir = out_dir.join("keys");
    fs::create_dir_all(&keys_dir).expect("create keys dir");
    let keys_dir_str = keys_dir.to_str().expect("keys dir str");

    let mut keygen_cmd = aletheia_cmd(&ws);
    keygen_cmd
        .args(["keygen", "--output", keys_dir_str, "--name", "test-key"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let keygen_out = keygen_cmd.output().expect("keygen output");
    assert!(
        keygen_out.status.success(),
        "keygen failed:\nstderr: {}",
        String::from_utf8_lossy(&keygen_out.stderr)
    );

    let sec_key = keys_dir.join("test-key.sec");
    let pub_key = keys_dir.join("test-key.pub");
    assert!(sec_key.exists(), "secret key file should exist");
    assert!(pub_key.exists(), "public key file should exist");

    // Validate key file: should be 64 hex chars (32 bytes).
    let sec_hex = fs::read_to_string(&sec_key).expect("read sec key");
    assert_eq!(
        sec_hex.trim().len(),
        64,
        "secret key should be 64 hex chars"
    );

    // ── 2. Capture ────────────────────────────────────────────────────────────
    let events = "tool_call: read file\ntool_call: write file\n";

    let mut capture_cmd = aletheia_cmd(&ws);
    capture_cmd
        .args(["capture", "--session", &session])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = capture_cmd.spawn().expect("spawn capture");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(events.as_bytes())
        .expect("write stdin");
    let cap_out = child.wait_with_output().expect("capture output");
    assert!(
        cap_out.status.success(),
        "capture failed:\nstderr: {}",
        String::from_utf8_lossy(&cap_out.stderr)
    );

    // ── 3. Seal with signing key ──────────────────────────────────────────────
    let pack_path = out_dir.join("signed.aletheia.json");
    let pack_path_str = pack_path.to_str().expect("pack path str");
    let sec_key_str = sec_key.to_str().expect("sec key str");

    let mut seal_cmd = aletheia_cmd(&ws);
    seal_cmd
        .args([
            "seal",
            "--session",
            &session,
            "--key",
            sec_key_str,
            "--output",
            pack_path_str,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let seal_out = seal_cmd.output().expect("seal output");
    assert!(
        seal_out.status.success(),
        "seal failed:\nstderr: {}",
        String::from_utf8_lossy(&seal_out.stderr)
    );

    // ── 4. Verify with public key ─────────────────────────────────────────────
    let pub_key_str = pub_key.to_str().expect("pub key str");

    let mut verify_cmd = aletheia_cmd(&ws);
    verify_cmd
        .args(["verify", pack_path_str, "--key", pub_key_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let verify_out = verify_cmd.output().expect("verify output");
    assert!(
        verify_out.status.success(),
        "verify failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&verify_out.stdout),
        String::from_utf8_lossy(&verify_out.stderr)
    );

    let verify_stdout = String::from_utf8_lossy(&verify_out.stdout);
    let verify_json: serde_json::Value =
        serde_json::from_str(&verify_stdout).expect("verify output is valid JSON");
    assert_eq!(verify_json["status"], "verified");
    assert_eq!(
        verify_json["signatures_valid"], 1,
        "should have 1 valid signature"
    );
    assert_eq!(verify_json["signatures_total"], 1);

    // ── Cleanup ───────────────────────────────────────────────────────────────
    cleanup_session(&session);
    let _ = fs::remove_dir_all(&out_dir);
}

/// Tampered pack must be rejected by verify with exit code 1.
#[test]
fn tampered_pack_fails_verify() {
    let ws = workspace_root();
    let session = unique_session("e2e-tampered");
    let out_dir = make_output_dir(&session);

    // Capture one event.
    let events = "important audit event\n";
    let mut cap_cmd = aletheia_cmd(&ws);
    cap_cmd
        .args(["capture", "--session", &session])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cap_cmd.spawn().expect("spawn capture");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(events.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap();

    // Seal into a pack.
    let pack_path = out_dir.join("tampered.aletheia.json");
    let pack_str = pack_path.to_str().unwrap();

    let mut seal_cmd = aletheia_cmd(&ws);
    seal_cmd
        .args(["seal", "--session", &session, "--output", pack_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    seal_cmd.output().unwrap();

    // Tamper: replace part of the payload in the on-disk JSON.
    let original = fs::read_to_string(&pack_path).expect("read pack");
    let tampered = original.replace("audit event", "TAMPERED event");
    assert_ne!(original, tampered, "tamper substitution had no effect");
    fs::write(&pack_path, tampered).expect("write tampered pack");

    // Verify must fail with exit code 1.
    let mut verify_cmd = aletheia_cmd(&ws);
    verify_cmd
        .args(["verify", pack_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let verify_out = verify_cmd.output().expect("verify output");
    assert!(
        !verify_out.status.success(),
        "verify should have failed on tampered pack"
    );
    assert_eq!(
        verify_out.status.code(),
        Some(1),
        "tampered pack should exit with code 1"
    );

    let stdout = String::from_utf8_lossy(&verify_out.stdout);
    assert!(
        stdout.contains("failed"),
        "expected 'failed' in output, got: {stdout}"
    );

    // ── Cleanup ───────────────────────────────────────────────────────────────
    cleanup_session(&session);
    let _ = fs::remove_dir_all(&out_dir);
}

/// Export produces valid markdown output with expected sections.
#[test]
fn export_markdown_format() {
    let ws = workspace_root();
    let session = unique_session("e2e-markdown");
    let out_dir = make_output_dir(&session);

    // Capture and seal a minimal pack.
    let events = "file read\nfile written\n";
    let mut cap_cmd = aletheia_cmd(&ws);
    cap_cmd
        .args(["capture", "--session", &session])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cap_cmd.spawn().expect("spawn");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(events.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap();

    let pack_path = out_dir.join("md-test.aletheia.json");
    let pack_str = pack_path.to_str().unwrap();

    let mut seal_cmd = aletheia_cmd(&ws);
    seal_cmd
        .args(["seal", "--session", &session, "--output", pack_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    seal_cmd.output().unwrap();

    // Export as markdown.
    let mut export_cmd = aletheia_cmd(&ws);
    export_cmd
        .args(["export", "--format", "markdown", pack_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let out = export_cmd.output().expect("export output");
    assert!(
        out.status.success(),
        "markdown export failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let md = String::from_utf8_lossy(&out.stdout);
    assert!(
        md.contains("# Aletheia Evidence Report"),
        "should have H1 heading"
    );
    assert!(
        md.contains("verified"),
        "should contain verification status"
    );
    assert!(
        md.contains("## Event Timeline"),
        "should have event timeline section"
    );
    assert!(
        md.contains("## Hash Chain"),
        "should have hash chain section"
    );

    // ── Cleanup ───────────────────────────────────────────────────────────────
    cleanup_session(&session);
    let _ = fs::remove_dir_all(&out_dir);
}
