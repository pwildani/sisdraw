use xy::XY;

/// A point in polar Theta-R space.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
pub struct TR {
    pub t: f64,
    pub r: f64,
}

impl TR {
    // Not from or into because this should be explicit.
    pub fn xy(self) -> XY {
        XY {
            x: self.t.cos() * self.r,
            y: self.t.sin() * self.r,
        }
    }

    pub fn interp_to(self, other: TR, n: f64) -> TR {
        TR {
            t: self.t + n * (other.t - self.t),
            r: self.r + n * (other.r - self.r),
        }
    }
}

impl Into<(f64, f64)> for TR {
    fn into(self) -> (f64, f64) {
        (self.t, self.r)
    }
}
