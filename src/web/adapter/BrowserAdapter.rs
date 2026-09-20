use std::io::Cursor;

use wasm_bindgen::prelude::*;

use crate::mapper::internaltonote::InternalToNoteMapper;
use crate::mapper::livetointernal::LiveToInternalMapper;
use crate::parser::live::LiveParser;
use crate::web::factory::projectpreviewfactory::ProjectPreviewFactory;

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct BrowserAdapter {
    liveMapper: LiveToInternalMapper,
    noteMapper: InternalToNoteMapper,
    previewFactory: ProjectPreviewFactory,
}

#[wasm_bindgen]
impl BrowserAdapter {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            liveMapper: LiveToInternalMapper::new(),
            noteMapper: InternalToNoteMapper::new(),
            previewFactory: ProjectPreviewFactory::new(),
        }
    }

    pub fn loadProject(&self, sourceBytes: &[u8]) -> Result<JsValue, JsValue> {
        let liveProject = LiveParser::new(Cursor::new(sourceBytes))
            .parse()
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let internalResult = self.liveMapper.map(&liveProject);
        let noteResult = self.noteMapper.map(&internalResult.value);
        let preview = self.previewFactory.create(&internalResult, &noteResult);

        serde_wasm_bindgen::to_value(&preview)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }
}
