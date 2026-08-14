use super::internal::{Beat, BeatRange};

#[test]
fn beat_range_rejects_backwards_and_non_finite_ranges() {
    assert!(
        BeatRange {
            start: Beat(0.0),
            end: Beat(4.0),
        }
        .is_valid()
    );
    assert!(
        !BeatRange {
            start: Beat(4.0),
            end: Beat(0.0),
        }
        .is_valid()
    );
    assert!(
        !BeatRange {
            start: Beat(0.0),
            end: Beat(f64::NAN),
        }
        .is_valid()
    );
}
