#![allow(non_snake_case)]

use std::env;
use std::error::Error;
use std::fs::{File, write};
use std::path::PathBuf;

use able_converter::exporter::note::NoteExporter;
use able_converter::mapper::internaltonote::InternalToNoteMapper;
use able_converter::mapper::livetointernal::LiveToInternalMapper;
use able_converter::model::internal::Diagnostic;
use able_converter::parser::live::LiveParser;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1).map(PathBuf::from);
    let sourcePath = arguments
        .next()
        .ok_or("usage: cargo run --example convert -- <project.als> <output.ablbundle>")?;
    let destinationPath = arguments
        .next()
        .ok_or("usage: cargo run --example convert -- <project.als> <output.ablbundle>")?;
    if arguments.next().is_some() {
        return Err("expected one .als path and one .ablbundle path".into());
    }

    let liveProject = LiveParser::new(File::open(sourcePath)?).parse()?;
    let internalResult = LiveToInternalMapper::new().map(&liveProject);
    printDiagnostics(&internalResult.diagnostics);
    if internalResult.hasErrors() {
        return Err("Live mapping stopped because error diagnostics were reported".into());
    }

    let noteResult = InternalToNoteMapper::new().map(&internalResult.value);
    printDiagnostics(&noteResult.diagnostics);
    if noteResult.hasErrors() {
        return Err("Note mapping stopped because error diagnostics were reported".into());
    }

    let bundle = NoteExporter::new().exportBundle(&noteResult.value)?;
    write(&destinationPath, bundle)?;
    println!("Wrote {}", destinationPath.display());
    Ok(())
}

fn printDiagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!(
            "{:?} {:?} [{}]: {}",
            diagnostic.severity, diagnostic.code, diagnostic.sourceId, diagnostic.message
        );
    }
}
