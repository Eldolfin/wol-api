use std::cmp::{self, Ordering};

#[derive(PartialEq, PartialOrd)]
pub struct ComparableFloats(f32);

impl From<f32> for ComparableFloats {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
impl Eq for ComparableFloats {}
impl Ord for ComparableFloats {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        debug_assert!(
            self.0.is_finite(),
            "Comparable floats can only compare finite numbers"
        );
        debug_assert!(
            other.0.is_finite(),
            "Comparable floats can only compare finite numbers"
        );
        if self < other {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}
