use tr::TR;
use xy::XY;

#[derive(Debug)]
pub struct ArcToXYIter {
    start: TR,
    end: TR,
    dims: (u32, u32),
    steps: u32,
    n: u32,
}

fn fmax(a: f64, b: f64) -> f64 {
    // std::cmp:max wants std::cmp::Ord, which f64 doesn't have.
    if a > b {
        return a;
    }
    return b;
}

impl ArcToXYIter {
    pub fn new(dims: (u32, u32), start: TR, end: TR) -> ArcToXYIter {
        let (w, h) = dims;
        let m = (w + h) as f64 / 2.0;
        let dpx = dist(start, end) * (m as f64);
        const PIXELS_PER_STEP: f64 = 2.5;
        let steps = fmax(1.0, dpx / PIXELS_PER_STEP) as u32;
        ArcToXYIter {
            start: start,
            end: end,
            dims: dims,
            steps: steps,
            n: 1,
        }
    }

    fn peek(&self) -> XY {
        self.start
            .interp_to(self.end, self.n as f64 / self.steps as f64)
            .xy()
    }
}

impl Iterator for ArcToXYIter {
    type Item = ((f32, f32), (f32, f32));

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }
        if self.n > self.steps {
            return None;
        }
        let last = self.peek();
        self.n += 1;
        let next = self.peek();
        return Some((last.on(self.dims), next.on(self.dims)));
    }
}

pub fn dist<T: Into<(f64, f64)>>(a: T, b: T) -> f64 {
    let (xa, ya) = a.into();
    let (xb, yb) = b.into();
    let x = xa - xb;
    let y = ya - yb;
    x.hypot(y)
}
