#![allow(non_snake_case)]

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};

use able_converter::model::electribe::{ElectribeFileKind, ElectribePattern, ElectribeProject};
use able_converter::model::live::LiveSetInspection;
use able_converter::parser::electribe::ElectribeParser;
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
            return Err("expected one .als, .e2spat, or .e2sallpat path".into());
        }
    }

    let path = path.ok_or(
        "usage: cargo run --example inspect -- <project.als|pattern.e2spat|all.e2sallpat> [--json]",
    )?;

    if isElectribeFile(&path) {
        let project = ElectribeParser::new(File::open(path)?).parse()?;
        if json {
            println!("{}", serde_json::to_string_pretty(&project)?);
        } else {
            printElectribeReport(&project);
        }
        return Ok(());
    }

    let report = LiveParser::new(File::open(path)?).inspect()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        printHumanReport(&report);
    }

    Ok(())
}

fn isElectribeFile(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("e2spat") || extension.eq_ignore_ascii_case("e2sallpat")
        })
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

fn printElectribeReport(project: &ElectribeProject) {
    let kind = match project.fileKind {
        ElectribeFileKind::Pattern => "pattern",
        ElectribeFileKind::AllPatterns => "all patterns",
    };
    println!("electribe sampler {kind} file ({})", project.product);
    println!("  Patterns: {}", project.patterns.len());
    for pattern in &project.patterns {
        printPattern(pattern);
    }
}

fn printPattern(pattern: &ElectribePattern) {
    let usedParts = pattern
        .parts
        .iter()
        .filter(|part| {
            part.steps
                .iter()
                .any(|step| step.noteSlots.iter().any(|&value| value != 0))
        })
        .map(|part| part.number.to_string())
        .collect::<Vec<_>>();
    println!(
        "  {:3} {:18} {:6.1} BPM  {} bar(s) {:?}  parts with notes: {}  motion slots: {}",
        pattern.number,
        pattern.name,
        f64::from(pattern.tempoTenths) / 10.0,
        pattern.lengthBars,
        pattern.beat,
        if usedParts.is_empty() {
            "none".to_owned()
        } else {
            usedParts.join(",")
        },
        pattern.motionSlots.len()
    );
}
