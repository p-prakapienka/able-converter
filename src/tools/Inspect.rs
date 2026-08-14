use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;

use able_converter::model::live::LiveSetInspection;
use able_converter::parser::live::inspect_als;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut path = None;
    let mut json = false;

    for argument in env::args_os().skip(1) {
        if argument == OsStr::new("--json") {
            json = true;
        } else if path.replace(PathBuf::from(argument)).is_some() {
            return Err("expected one .als path".into());
        }
    }

    let path = path.ok_or("usage: cargo run --example inspect -- <project.als> [--json]")?;
    let report = inspect_als(File::open(path)?)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human_report(&report);
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
