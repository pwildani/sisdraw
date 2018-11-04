extern crate image;
extern crate imageproc;

use image::*;
use imageproc::drawing::draw_antialiased_line_segment_mut;
use imageproc::pixelops::interpolate;

use std::cmp::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct XY {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TR {
    t: f32,
    r: f32,
}

impl TR {
    pub fn xy(self) -> XY {
        XY {
            x: self.t.cos() * self.r,
            y: self.t.sin() * self.r,
        }
    }

    pub fn interp_to(self, other: TR, n: f32) -> TR {
        TR {
            t: self.t + n * (other.t - self.t),
            r: self.r + n * (other.r - self.r),
        }
    }
}

impl Into<(f32, f32)> for XY {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl Into<(f32, f32)> for TR {
    fn into(self) -> (f32, f32) {
        (self.t, self.r)
    }
}

pub fn xy_dist(a: TR, b: TR) -> f32 {
    dist(a.xy(), b.xy())
}

pub fn dist<T: Into<(f32, f32)>>(a: T, b: T) -> f32 {
    let (xa, ya) = a.into();
    let (xb, yb) = b.into();
    let x = xa - xb;
    let y = ya - yb;
    let sq = x * x + y * y;
    sq.sqrt()
}

impl XY {
    fn on_img<I: GenericImage>(&self, img: &I) -> (i32, i32) {
        self.on(img.dimensions())
    }

    fn on(&self, dims: (u32, u32)) -> (i32, i32) {
        let (w, h) = dims;
        let w2 = w as f32 / 2.0;
        let h2 = h as f32 / 2.0;
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
    incr: f32,
    s: f32,
}

impl ArcToXYIter {
    pub fn new(dims: (u32, u32), start: TR, end: TR) -> ArcToXYIter {
        let (w, h) = dims;
        let m = (w + h) as f32 / 2.0;
        let steps = (dist(start, end) * (m as f32)) as u32 / 10;
        ArcToXYIter {
            start: start,
            end: end,
            last: start.xy(),
            dims: dims,
            steps: steps,
            incr: 1.0 / steps as f32,
            s: 0.0,
        }
    }

    fn peek(&self) -> XY {
        self.start.interp_to(self.end, self.s + self.incr).xy()
    }
}

impl Iterator for ArcToXYIter {
    type Item = ((i32, i32), (i32, i32));

    fn next(&mut self) -> Option<Self::Item> {
        let s = self.s;
        if s > 1.0 {
            return None;
        }

        let mut next = self.peek();
        let m = (self.dims.0 + self.dims.1) as f32 / 2.0;
        // Try to ensure that the next segment can be approximated with a straight line by not
        // having too many (or too few!) pixels.
        if s != 1.0 {
            while dist(self.last, next) * m > 8.0 {
                self.incr /= 2.0;
                next = self.peek();
            }
            while dist(self.last, next) * m < 3.0 {
                self.incr *= 2.0;
                next = self.peek();
            }
        }

        if s == 1.0 || self.s + self.incr < 1.0 {
            self.s += self.incr;
        } else {
            self.incr = s;
            self.s = 1.0;
        }
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
        draw_antialiased_line_segment_mut(img, s1, s2, color, &blend);
    }
}

fn main() {
    println!("Hello, world!");

    let mut img = GrayImage::from_pixel(512, 512, Luma([255u8]));

    draw_antialiased_line_segment_mut(&mut img, (3, 3), (12, 24), Luma([128u8]), interpolate);

    draw_arc(
        &mut img,
        TR { r: 0.1, t: 0.0 },
        TR {
            r: 1.0,
            t: 100.0 * 3.14,
        },
        Luma([128u8]),
        interpolate,
    );

    img.save("out.png").unwrap();
}
