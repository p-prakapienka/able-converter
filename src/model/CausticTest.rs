use super::caustic::{CausticControl, CausticControlCollection, CausticSectionTag};

#[test]
fn sectionTagRoundTripsKnownChunkIdentifiers() {
    assert_eq!(
        CausticSectionTag::fromBytes(b"OUTP"),
        Some(CausticSectionTag::OutputPanel)
    );
    assert_eq!(
        CausticSectionTag::fromBytes(b"EFFX"),
        Some(CausticSectionTag::Effects)
    );
    assert_eq!(
        CausticSectionTag::fromBytes(b"MIXR"),
        Some(CausticSectionTag::Mixer)
    );
    assert_eq!(
        CausticSectionTag::fromBytes(b"MSTR"),
        Some(CausticSectionTag::Master)
    );
    assert_eq!(
        CausticSectionTag::fromBytes(b"SEQN"),
        Some(CausticSectionTag::Sequencer)
    );
    assert_eq!(CausticSectionTag::fromBytes(b"WXYZ"), None);
    assert_eq!(CausticSectionTag::OutputPanel.asBytes(), b"OUTP");
}

#[test]
fn controlCollectionReturnsTheValueForAnIdentifier() {
    let collection = CausticControlCollection {
        controls: vec![
            CausticControl { id: 1, value: 0.25 },
            CausticControl { id: 7, value: -1.5 },
        ],
    };

    assert_eq!(collection.valueOf(1), Some(0.25));
    assert_eq!(collection.valueOf(7), Some(-1.5));
    assert_eq!(collection.valueOf(99), None);
}
