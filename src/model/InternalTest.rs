use super::internal::{
    Beat, BeatRange, Diagnostic, DiagnosticCode, DiagnosticSeverity, MappingResult,
};

#[test]
fn beatRangeRejectsBackwardsOrNonFiniteRanges() {
    assert!(
        BeatRange {
            start: Beat(0.0),
            end: Beat(4.0),
        }
        .isValid()
    );
    assert!(
        !BeatRange {
            start: Beat(4.0),
            end: Beat(0.0),
        }
        .isValid()
    );
    assert!(
        !BeatRange {
            start: Beat(0.0),
            end: Beat(f64::NAN),
        }
        .isValid()
    );
}

#[test]
fn mappingResultReportsWhetherItContainsErrors() {
    let result = MappingResult {
        value: (),
        diagnostics: vec![Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode::InvalidNote,
            sourceId: "note-1".to_owned(),
            message: "invalid note".to_owned(),
        }],
    };

    assert!(result.hasErrors());
}
