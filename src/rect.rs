use {
    serde::{Serialize, Deserialize},
    crate::Coord
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub top: Coord,
    pub left: Coord,
    pub bottom: Coord,
    pub right: Coord
}

impl Rect {
    pub fn intersects_with(&self, other: &Rect) -> bool {
        let max_left = std::cmp::max(self.left, other.left);
        let min_right = std::cmp::min(self.right, other.right);
        let max_top = std::cmp::max(self.top, other.top);
        let min_bottom = std::cmp::min(self.bottom, other.bottom);

        max_left < min_right && max_top < min_bottom
    }
}