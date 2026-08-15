#![allow(non_snake_case)]

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;

use able_converter::model::live::LiveSetInspection;
use able_converter::parser::live::LiveParser;

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
    let report = LiveParser::new(File::open(path)?).inspect()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        printHumanReport(&report);
    }

    Ok(())
}

fn printHumanReport(report: &LiveSetInspection) {
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
    println!("    Return: {}", report.tracks.returnTracks);
    println!("    Main/Master: {}", report.tracks.main);
    println!("  MIDI clips:");
    println!("    Session: {}", report.clips.sessionMidi);
    println!("    Arrangement: {}", report.clips.arrangementMidi);
    println!("    Unclassified: {}", report.clips.unclassifiedMidi);
    println!("  Audio clips:");
    println!("    Session: {}", report.clips.sessionAudio);
    println!("    Arrangement: {}", report.clips.arrangementAudio);
    println!("    Unclassified: {}", report.clips.unclassifiedAudio);
}
