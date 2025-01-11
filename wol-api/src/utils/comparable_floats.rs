use std::cmp::Ordering;

#[derive(PartialEq, PartialOrd)]
pub struct ComparableFloats(f32);

impl From<f32> for ComparableFloats {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
impl Eq for ComparableFloats {}
impl Ord for ComparableFloats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        debug_assert!(self.0.is_finite());
        debug_assert!(other.0.is_finite());
        if self < other {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}
