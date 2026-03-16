mod cmd_capture;
mod cmd_export;
mod cmd_keygen;
mod cmd_seal;
mod cmd_verify;
mod paths;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "aletheia",
    version = "0.1.0",
    about = "Aletheia cryptographic evidence packs"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an Ed25519 keypair
    Keygen {
        /// Directory to write keys into (default: platform config dir)
        #[arg(long)]
        output: Option<String>,
        /// Base name for the key files
        #[arg(long, default_value = "default")]
        name: String,
    },
    /// Capture stdin events into a session
    Capture {
        /// Session name
        #[arg(long)]
        session: String,
        /// Event source label
        #[arg(long, default_value = "manual")]
        source: String,
        /// Comma-separated key=value context pairs (e.g. repo=X,branch=Y,pr=N)
        #[arg(long)]
        context: Option<String>,
    },
    /// Seal a session into an evidence pack
    Seal {
        /// Session name
        #[arg(long)]
        session: String,
        /// Path to signing key (.sec file)
        #[arg(long)]
        key: Option<String>,
        /// Output path for the pack
        #[arg(long)]
        output: Option<String>,
    },
    /// Verify an evidence pack
    Verify {
        /// Path to the evidence pack
        pack: String,
        /// Path to verifying key (.pub file)
        #[arg(long)]
        key: Option<String>,
    },
    /// Export an evidence pack in a given format
    Export {
        /// Output format (e.g. json, csv)
        #[arg(long)]
        format: String,
        /// Path to the evidence pack
        pack: String,
        /// Output path
        #[arg(long)]
        output: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Keygen { output, name } => cmd_keygen::run(output, name),
        Commands::Capture {
            session,
            source,
            context,
        } => cmd_capture::run(session, source, context),
        Commands::Seal {
            session,
            key,
            output,
        } => cmd_seal::run(session, key, output),
        Commands::Verify { pack, key } => cmd_verify::run(pack, key),
        Commands::Export {
            format,
            pack,
            output,
        } => cmd_export::run(format, pack, output),
    };

    if let Err(e) = result {
        let msg = e.to_string();
        eprintln!("Error: {msg}");
        let code =
            if msg.contains("integrity") || msg.contains("mismatch") || msg.contains("signature") {
                1
            } else {
                3
            };
        std::process::exit(code);
    }
}
