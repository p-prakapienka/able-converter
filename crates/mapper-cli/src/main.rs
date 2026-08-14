use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use mapper_model::LiveSetInspection;

#[derive(Debug, Parser)]
#[command(name = "able-converter", version, about = "Inspect and convert Ableton projects")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect the high-level structure of an Ableton Live Set.
    Inspect {
        /// Path to a gzip-compressed Ableton Live Set.
        path: PathBuf,
        /// Emit the report as JSON.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Inspect { path, json } => {
            let report = mapper_core::inspect_als_path(path)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_human_report(&report);
            }
        }
    }

    Ok(())
}

fn print_human_report(report: &LiveSetInspection) {
    let version = report.format.minor.as_deref().unwrap_or("unknown");
    let creator = report.format.creator.as_deref().unwrap_or("unknown");
    let tempo = report
        .tempo
        .map_or_else(|| "unknown".to_owned(), |value| format!("{value} BPM"));

    println!("Ableton Live Set");
    println!("  Format: {version}");
    println!("  Creator: {creator}");
    println!("  Tempo: {tempo}");
    println!("  Tracks:");
    println!("    MIDI: {}", report.tracks.midi);
    println!("    Audio: {}", report.tracks.audio);
    println!("    Group: {}", report.tracks.group);
    println!("    Return: {}", report.tracks.return_tracks);
    println!("    Main/Master: {}", report.tracks.main);
    println!("  MIDI clips:");
    println!("    Session: {}", report.clips.session_midi);
    println!("    Arrangement: {}", report.clips.arrangement_midi);
    println!("    Unclassified: {}", report.clips.unclassified_midi);
    println!("  Audio clips:");
    println!("    Session: {}", report.clips.session_audio);
    println!("    Arrangement: {}", report.clips.arrangement_audio);
    println!("    Unclassified: {}", report.clips.unclassified_audio);
}
