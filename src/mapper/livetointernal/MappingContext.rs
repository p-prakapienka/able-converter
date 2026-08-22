use crate::model::internal::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, MappingResult, Project,
};

pub(super) struct LiveToInternalMappingContext {
    diagnostics: Vec<Diagnostic>,
}

impl LiveToInternalMappingContext {
    pub(super) fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub(super) fn warning(
        &mut self,
        sourceId: String,
        code: DiagnosticCode,
        message: impl Into<String>,
    ) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code,
            sourceId,
            message: message.into(),
        });
    }

    pub(super) fn error(
        &mut self,
        sourceId: String,
        code: DiagnosticCode,
        message: impl Into<String>,
    ) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            code,
            sourceId,
            message: message.into(),
        });
    }

    pub(super) fn finish(self, value: Project) -> MappingResult<Project> {
        MappingResult {
            value,
            diagnostics: self.diagnostics,
        }
    }
}
