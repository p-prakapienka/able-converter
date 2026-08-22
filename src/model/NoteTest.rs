use super::note::{NOTE_SCHEMA_URI, NoteProject};

#[test]
fn deserializesSchemaPropertyIntoTheTypedModel() {
    let json = format!(
        r#"{{
            "$schema":"{NOTE_SCHEMA_URI}",
            "stepEditorResolution":"1/16",
            "tempo":120.0,
            "globalGrooveAmount":0.0,
            "timeSignature":{{"upper":4,"lower":4}},
            "rootNote":0,
            "scale":"major",
            "melodicLayout":"chromatic",
            "tracks":[],
            "returnTracks":[],
            "masterTrack":{{
                "color":23,
                "isSelected":false,
                "devices":[],
                "mixer":{{"pan":0.0,"volume":0.0}}
            }},
            "scenes":[],
            "grooves":[],
            "metadata":{{"usedFeatures":[]}}
        }}"#
    );

    let project: NoteProject = serde_json::from_str(&json).expect("valid Note project");

    assert_eq!(project.schema, NOTE_SCHEMA_URI);
}
