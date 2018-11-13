use std::cmp::max;
use tr::TR;
use xy::XY;

#[derive(Debug)]
pub struct ArcToXYIter {
    start: TR,
    end: TR,
    last: XY,
    dims: (u32, u32),
    steps: u32,
    incr: f64,
    s: f64,
    done: bool,
}

impl ArcToXYIter {
    pub fn new(dims: (u32, u32), start: TR, end: TR) -> ArcToXYIter {
        let (w, h) = dims;
        let m = (w + h) as f64 / 2.0;
        let steps = max(1, (dist(start, end) * (m as f64)) as u32 / 10);
        ArcToXYIter {
            start: start,
            end: end,
            last: start.xy(),
            dims: dims,
            steps: steps,
            incr: 1.0 / steps as f64,
            s: 0.0,
            done: false,
        }
    }

    fn peek(&self) -> XY {
        self.start.interp_to(self.end, self.s + self.incr).xy()
    }

    fn default_incr(&self) -> f64 {
        1.0 / self.steps as f64
    }
}

impl Iterator for ArcToXYIter {
    type Item = ((f32, f32), (f32, f32));

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }
        let s = self.s;
        if self.done {
            return None;
        }
        if s > 1.0 {
            self.done = true;
        }
        //println!("{:?}", self);

        let mut next = self.peek();
        let m = (self.dims.0 + self.dims.1) as f64 / 2.0;
        // Try to ensure that the next segment can be approximated with a straight line by not
        // having too many (or too few!) pixels.
        if s != 1.0 {
            while dist(self.last, next) * m > 8.0 {
                self.incr *= 7.9 / (dist(self.last, next) * m);
                //println!("Adj too far: incr -> {}, dist={}", self.incr, dist(self.last, next)*m);
                next = self.peek();
            }
            while dist(self.last, next) * m < 3.0 {
                self.incr *= 3.1 / (dist(self.last, next) * m);
                //println!("Adj too short: incr -> {}, dist={}", self.incr, dist(self.last, next)*m);
                next = self.peek();
            }
            if self.incr <= 0.0 {
                println!("Bad incr!");
                self.incr = self.default_incr();
            }
            if self.incr == std::f64::INFINITY {
                println!(
                    "Infinite incr, reset to {}, steps={}",
                    self.default_incr(),
                    self.steps
                );
                self.incr = self.default_incr();
                next = self.peek();
            }
            if self.last == next {
                self.s += self.incr;
                return self.next();
            }
        }

        //println!("s {}, incr {}", self.s, self.incr);
        if s == 1.0 || self.s + self.incr < 1.0 {
            self.s += self.incr;
        } else {
            self.incr = 1.0;
            self.s = 1.0;
            self.done = true;
        }
        //println!("-> s {}, incr {}", self.s, self.incr);
        let a = self.last.on(self.dims);
        let b = next.on(self.dims);
        self.last = next;
        return Some((a, b));
    }
}

/*pub fn xy_dist(a: TR, b: TR) -> f64 {
    dist(a.xy(), b.xy())
}
*/

pub fn dist<T: Into<(f64, f64)>>(a: T, b: T) -> f64 {
    let (xa, ya) = a.into();
    let (xb, yb) = b.into();
    let x = xa - xb;
    let y = ya - yb;
    let sq = x * x + y * y;
    sq.sqrt()
}
