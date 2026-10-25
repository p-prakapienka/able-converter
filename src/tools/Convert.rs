#![allow(non_snake_case)]

use std::env;
use std::error::Error;
use std::fs::{read, write};
use std::path::PathBuf;

use able_converter::exporter::note::NoteExporter;
use able_converter::mapper::arrangementtosession::ArrangementToSessionMapper;
use able_converter::mapper::energyxttointernal::EnergyXtToInternalMapper;
use able_converter::mapper::internaltonote::InternalToNoteMapper;
use able_converter::mapper::livetointernal::LiveToInternalMapper;
use able_converter::model::internal::{Diagnostic, MappingResult, Project, TrackKind};
use able_converter::parser::energyxt::EnergyXtParser;
use able_converter::parser::live::LiveParser;

const USAGE: &str = "usage: cargo run --example convert -- <project.als|project.xt> \
                     <output.ablbundle> [--scenes 0,1,2]";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut paths = Vec::new();
    let mut selectedScenes = None;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--scenes" {
            let list = arguments.next().ok_or(USAGE)?;
            selectedScenes = Some(parseSceneList(&list)?);
        } else {
            paths.push(PathBuf::from(argument));
        }
    }
    let [sourcePath, destinationPath] = <[PathBuf; 2]>::try_from(paths).map_err(|_| USAGE)?;

    let bytes = read(&sourcePath)?;
    let internalResult = if EnergyXtParser::<&[u8]>::looksLikeProject(&bytes) {
        mapEnergyXt(&bytes)?
    } else {
        let liveProject = LiveParser::new(bytes.as_slice()).parse()?;
        LiveToInternalMapper::new().map(&liveProject)
    };
    printDiagnostics(&internalResult.diagnostics);
    if internalResult.hasErrors() {
        return Err("source mapping stopped because error diagnostics were reported".into());
    }

    let project = &internalResult.value;
    let noteMapper = InternalToNoteMapper::new();
    let noteResult = match selectedScenes {
        Some(scenes) => noteMapper.mapSelected(project, midiTrackIds(project), scenes),
        None => noteMapper.map(project),
    };
    printDiagnostics(&noteResult.diagnostics);
    if noteResult.hasErrors() {
        printScenes(project);
        return Err("Note mapping stopped because error diagnostics were reported".into());
    }

    let bundle = NoteExporter::new().exportBundle(&noteResult.value)?;
    write(&destinationPath, bundle)?;
    println!("Wrote {}", destinationPath.display());
    Ok(())
}

/// energyXT has only a timeline, so its Arrangement clips are laid out as scenes.
fn mapEnergyXt(bytes: &[u8]) -> Result<MappingResult<Project>, Box<dyn Error>> {
    let source = EnergyXtParser::new(bytes).parse()?;
    let arrangement = EnergyXtToInternalMapper::new().map(&source);
    if arrangement.hasErrors() {
        return Ok(arrangement);
    }
    let mut session = ArrangementToSessionMapper::new().map(&arrangement.value);
    let mut diagnostics = arrangement.diagnostics;
    diagnostics.append(&mut session.diagnostics);
    Ok(MappingResult {
        value: session.value,
        diagnostics,
    })
}

fn parseSceneList(list: &str) -> Result<Vec<usize>, Box<dyn Error>> {
    list.split(',')
        .map(|item| {
            item.trim()
                .parse::<usize>()
                .map_err(|_| format!("invalid scene index '{item}'").into())
        })
        .collect()
}

fn midiTrackIds(project: &Project) -> Vec<String> {
    project
        .tracks
        .iter()
        .filter(|track| track.kind == TrackKind::Midi)
        .map(|track| track.id.clone())
        .collect()
}

fn printScenes(project: &Project) {
    eprintln!("Scenes available for --scenes:");
    for scene in &project.scenes {
        eprintln!("  {}: {}", scene.index, scene.name);
    }
}

fn printDiagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!(
            "{:?} {:?} [{}]: {}",
            diagnostic.severity, diagnostic.code, diagnostic.sourceId, diagnostic.message
        );
    }
}
