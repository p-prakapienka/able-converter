#![allow(non_snake_case)]

use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::{File, write};
use std::path::{Path, PathBuf};

use able_converter::exporter::note::NoteExporter;
use able_converter::mapper::electribetointernal::ElectribeToInternalMapper;
use able_converter::mapper::internaltonote::InternalToNoteMapper;
use able_converter::mapper::livetointernal::LiveToInternalMapper;
use able_converter::model::electribe::ElectribeFileKind;
use able_converter::model::internal::{Diagnostic, MappingResult, Project};
use able_converter::parser::electribe::ElectribeParser;
use able_converter::parser::live::LiveParser;

const USAGE: &str = "usage: cargo run --example convert -- <source> <output.ablbundle> \
    [--patterns 1-4,7] [--parts 1,2,5-8]\n  \
    <source> is an Ableton Live .als, an electribe sampler .e2spat, or an .e2sallpat file;\n  \
    --patterns and --parts apply to electribe sources only";

struct Options {
    sourcePath: PathBuf,
    destinationPath: PathBuf,
    patterns: Option<Vec<usize>>,
    parts: Option<Vec<usize>>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let options = parseOptions()?;
    let (internalResult, selection) = if isElectribeFile(&options.sourcePath) {
        mapElectribe(&options)?
    } else {
        if options.patterns.is_some() || options.parts.is_some() {
            return Err("--patterns and --parts apply to electribe sources only".into());
        }
        let liveProject = LiveParser::new(File::open(&options.sourcePath)?).parse()?;
        (LiveToInternalMapper::new().map(&liveProject), None)
    };

    printDiagnostics(&internalResult.diagnostics);
    if internalResult.hasErrors() {
        return Err("source mapping stopped because error diagnostics were reported".into());
    }

    let noteMapper = InternalToNoteMapper::new();
    let noteResult = match selection {
        Some((trackIds, sceneIndices)) => {
            noteMapper.mapSelected(&internalResult.value, trackIds, sceneIndices)
        }
        None => noteMapper.map(&internalResult.value),
    };
    printDiagnostics(&noteResult.diagnostics);
    if noteResult.hasErrors() {
        return Err("Note mapping stopped because error diagnostics were reported".into());
    }

    let bundle = NoteExporter::new().exportBundle(&noteResult.value)?;
    write(&options.destinationPath, bundle)?;
    println!("Wrote {}", options.destinationPath.display());
    Ok(())
}

type Selection = Option<(Vec<String>, Vec<usize>)>;

fn mapElectribe(options: &Options) -> Result<(MappingResult<Project>, Selection), Box<dyn Error>> {
    let source = ElectribeParser::new(File::open(&options.sourcePath)?).parse()?;
    let patterns = match (&options.patterns, source.fileKind) {
        (Some(patterns), _) => patterns.clone(),
        (None, ElectribeFileKind::Pattern) => vec![1],
        (None, ElectribeFileKind::AllPatterns) => {
            return Err("an .e2sallpat holds 250 patterns; choose up to 8 with --patterns".into());
        }
    };

    let internalResult = ElectribeToInternalMapper::new().mapPatterns(&source, &patterns);
    let parts = options.parts.clone().unwrap_or_else(|| {
        let mut used = internalResult
            .value
            .midiClips
            .iter()
            .filter_map(|clip| clip.id.rsplit(':').next()?.parse::<usize>().ok())
            .collect::<Vec<_>>();
        used.sort_unstable();
        used.dedup();
        used
    });
    let trackIds = parts.iter().map(|part| format!("part:{part}")).collect();
    let sceneIndices = patterns.iter().map(|pattern| pattern - 1).collect();
    Ok((internalResult, Some((trackIds, sceneIndices))))
}

fn parseOptions() -> Result<Options, Box<dyn Error>> {
    let mut paths = Vec::new();
    let mut patterns = None;
    let mut parts = None;
    let mut arguments = env::args_os().skip(1);

    while let Some(argument) = arguments.next() {
        if argument == OsStr::new("--patterns") {
            patterns = Some(parseNumberList(arguments.next(), "--patterns", 250)?);
        } else if argument == OsStr::new("--parts") {
            parts = Some(parseNumberList(arguments.next(), "--parts", 16)?);
        } else {
            paths.push(PathBuf::from(argument));
        }
    }

    let [sourcePath, destinationPath]: [PathBuf; 2] =
        paths.try_into().map_err(|_| USAGE.to_owned())?;
    Ok(Options {
        sourcePath,
        destinationPath,
        patterns,
        parts,
    })
}

/// Parse a list such as `1-4,7` into one-based numbers no larger than `maximum`.
fn parseNumberList(
    value: Option<std::ffi::OsString>,
    option: &str,
    maximum: usize,
) -> Result<Vec<usize>, Box<dyn Error>> {
    let value = value
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| format!("{option} expects a list such as 1-4,7"))?;
    let mut numbers = Vec::new();

    for item in value.split(',') {
        let (start, end) = item.split_once('-').unwrap_or((item, item));
        let start = start.trim().parse::<usize>()?;
        let end = end.trim().parse::<usize>()?;
        if start == 0 || end < start || end > maximum {
            return Err(format!("{option} entry {item:?} must be within 1..={maximum}").into());
        }
        numbers.extend(start..=end);
    }

    Ok(numbers)
}

fn isElectribeFile(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("e2spat") || extension.eq_ignore_ascii_case("e2sallpat")
        })
}

fn printDiagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!(
            "{:?} {:?} [{}]: {}",
            diagnostic.severity, diagnostic.code, diagnostic.sourceId, diagnostic.message
        );
    }
}
