use aletheia_core::chain::HashChain;
use aletheia_core::event::{Event, EventContext};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};

/// Parse --context "repo=X,branch=Y,pr=N" into (repo, branch, pr_number).
fn parse_context(ctx: &str) -> (Option<String>, Option<String>, Option<u64>) {
    let mut repo = None;
    let mut branch = None;
    let mut pr_number = None;

    for pair in ctx.split(',') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("").trim();
        let val = parts.next().unwrap_or("").trim();
        match key {
            "repo" => repo = Some(val.to_string()),
            "branch" => branch = Some(val.to_string()),
            "pr" => pr_number = val.parse::<u64>().ok(),
            _ => {}
        }
    }

    (repo, branch, pr_number)
}

pub fn run(
    session: String,
    source: String,
    context: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse context key=value pairs
    let (repo, branch, pr_number) = context
        .as_deref()
        .map(parse_context)
        .unwrap_or((None, None, None));

    // Create session directory
    let session_dir = crate::paths::session_dir(&session);
    fs::create_dir_all(&session_dir)?;

    // Write metadata.json
    let metadata = serde_json::json!({
        "session_id": &session,
        "source": &source,
        "repo": repo,
        "branch": branch,
        "pr_number": pr_number,
    });
    let meta_path = session_dir.join("metadata.json");
    fs::write(&meta_path, serde_json::to_string_pretty(&metadata)?)?;

    // Open receipts.jsonl in append mode
    let receipts_path = session_dir.join("receipts.jsonl");
    let mut receipts_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&receipts_path)?;

    let mut chain = HashChain::new();
    let stdin = io::stdin();
    let mut count = 0usize;

    for line_result in stdin.lock().lines() {
        let line = line_result?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse the event
        let mut event = if trimmed.starts_with('{') {
            Event::from_json_line(trimmed, &session)
                .unwrap_or_else(|_| Event::from_plain_text(trimmed, &session))
        } else {
            Event::from_plain_text(trimmed, &session)
        };

        // Override source from CLI arg
        event.source = source.clone();

        // Override context fields from parsed CLI --context
        event.context = EventContext {
            session_id: session.clone(),
            repo: repo.clone(),
            branch: branch.clone(),
            pr_number,
            tool: event.context.tool,
            policy: event.context.policy,
            result: event.context.result,
        };

        // Append to chain and write receipt
        let receipt = chain.append(event)?;
        let json_line = serde_json::to_string(&receipt)?;
        writeln!(receipts_file, "{}", json_line)?;

        count += 1;
    }

    eprintln!("Captured {} event(s) into session '{}'", count, session);
    eprintln!("  Receipts: {}", receipts_path.display());

    Ok(())
}
