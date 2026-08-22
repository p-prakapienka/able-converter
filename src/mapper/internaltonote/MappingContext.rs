use crate::model::internal::{Diagnostic, DiagnosticCode, DiagnosticSeverity, MappingResult};
use crate::model::note::NoteProject;

pub(super) struct InternalToNoteMappingContext {
    selectedTrackIds: Vec<String>,
    selectedSceneIndices: Vec<usize>,
    diagnostics: Vec<Diagnostic>,
}

impl InternalToNoteMappingContext {
    pub(super) fn new(selectedTrackIds: Vec<String>, selectedSceneIndices: Vec<usize>) -> Self {
        Self {
            selectedTrackIds,
            selectedSceneIndices,
            diagnostics: Vec::new(),
        }
    }

    pub(super) fn selectedTrackIds(&self) -> &[String] {
        &self.selectedTrackIds
    }

    pub(super) fn selectedSceneIndices(&self) -> &[usize] {
        &self.selectedSceneIndices
    }

    pub(super) fn warning(
        &mut self,
        sourceId: impl Into<String>,
        code: DiagnosticCode,
        message: impl Into<String>,
    ) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code,
            sourceId: sourceId.into(),
            message: message.into(),
        });
    }

    pub(super) fn error(
        &mut self,
        sourceId: impl Into<String>,
        code: DiagnosticCode,
        message: impl Into<String>,
    ) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code,
            sourceId: sourceId.into(),
            message: message.into(),
        });
    }

    pub(super) fn finish(self, value: NoteProject) -> MappingResult<NoteProject> {
        MappingResult {
            value,
            diagnostics: self.diagnostics,
        }
    }
}
