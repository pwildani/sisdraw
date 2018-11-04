extern crate csv;
extern crate image;
extern crate imageproc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::fs::File;
use std::process;

use image::*;
use imageproc::drawing::draw_antialiased_line_segment_mut;
use imageproc::pixelops::interpolate;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct XY {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
pub struct TR {
    t: f64,
    r: f64,
}

impl TR {
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

impl Into<(f64, f64)> for XY {
    fn into(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl Into<(f64, f64)> for TR {
    fn into(self) -> (f64, f64) {
        (self.t, self.r)
    }
}

pub fn xy_dist(a: TR, b: TR) -> f64 {
    dist(a.xy(), b.xy())
}

pub fn dist<T: Into<(f64, f64)>>(a: T, b: T) -> f64 {
    let (xa, ya) = a.into();
    let (xb, yb) = b.into();
    let x = xa - xb;
    let y = ya - yb;
    let sq = x * x + y * y;
    sq.sqrt()
}

impl XY {
    fn on(&self, dims: (u32, u32)) -> (i32, i32) {
        let (w, h) = dims;
        let w2 = w as f64 / 2.0;
        let h2 = h as f64 / 2.0;
        ((w2 + self.x * w2) as i32, (h2 + self.y * h2) as i32)
    }
}

#[derive(Debug)]
struct ArcToXYIter {
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
        let steps = (dist(start, end) * (m as f64)) as u32 / 10;
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
}

impl Iterator for ArcToXYIter {
    type Item = ((i32, i32), (i32, i32));

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
                self.incr /= 2.0;
                next = self.peek();
                //println!("Adj: incr -> {}, dist={}", self.incr, dist(self.last, next)*m);
            }
            while dist(self.last, next) * m < 3.0 {
                self.incr *= 2.0;
                next = self.peek();
                //println!("Adj: incr -> {}, dist={}", self.incr, dist(self.last, next)*m);
            }
            if self.incr <= 0.0 {
                panic!("Bad incr!");
            }
            if self.incr == std::f64::INFINITY {
                self.incr = 100000.0;
                next = self.peek();
            }
            if self.last == next {
                panic!("no progress!");
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

fn draw_arc<I, B>(img: &mut I, start: TR, end: TR, color: I::Pixel, blend: B)
where
    I: GenericImage,
    I::Pixel: 'static,
    B: Fn(I::Pixel, I::Pixel, f32) -> I::Pixel,
{
    for (s1, s2) in ArcToXYIter::new(img.dimensions(), start, end) {
        println!("Draw {:?} -> {:?}", s1, s2);
        draw_antialiased_line_segment_mut(img, s1, s2, color, &blend);
    }
}

fn run() -> Result<(), Box<std::error::Error>> {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let mut img = GrayImage::from_pixel(2000, 2000, Luma([255u8]));
    let f = File::open(args[1].clone()).expect("file not found");
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b' ')
        .from_reader(f);
    let mut iter = rdr.deserialize();
    let first: TR = iter.next().ok_or("missing first record").unwrap().unwrap();
    let mut last: TR = iter.next().ok_or("missing second record").unwrap().unwrap();
    for rec in iter {
        let point: TR = rec?;
        println!("TR {:?} -> {:?}", last, point);
        draw_arc(&mut img, last, point, Luma([128u8]), interpolate);
        last = point;
    }

    img.save("out.png")?;
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
