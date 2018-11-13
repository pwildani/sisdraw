/// A point in cartesian X-Y space.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct XY {
    pub x: f64,
    pub y: f64,
}

impl Into<(f64, f64)> for XY {
    fn into(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl XY {
    /// Apply this point to the given units.
    pub fn on(&self, dims: (u32, u32)) -> (f32, f32) {
        let (w, h) = dims;
        let w2 = w as f64 / 2.0;
        let h2 = h as f64 / 2.0;
        ((w2 + self.x * w2) as f32, (h2 + self.y * h2) as f32)
    }
}
