use super::internal::{Beat, BeatRange};

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
