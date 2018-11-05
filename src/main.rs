extern crate csv;
extern crate image;
extern crate imageproc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::cmp::max;
use std::env;
use std::error::Error;
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

fn draw_fat_arc<I, B>(
    img: &mut I,
    start: TR,
    end: TR,
    bgcolor: I::Pixel,
    centercolor: I::Pixel,
    blend: B,
) where
    I: GenericImage,
    I::Pixel: 'static,
    B: Fn(I::Pixel, I::Pixel, f32) -> I::Pixel,
{
    for (s1, s2) in ArcToXYIter::new(img.dimensions(), start, end) {
        // println!("Draw {:?} -> {:?}", s1, s2);
        let (x1, y1) = s1;
        let (x2, y2) = s2;
        let slope = (x2 - x1) as f32 / (y2 - y1) as f32;

        if -1.0 <= slope && slope < 1.0 {
            draw_antialiased_line_segment_mut(img, (x1 - 3, y1), (x2 - 3, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 + 3, y1), (x2 + 3, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 - 2, y1), (x2 - 2, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 + 2, y1), (x2 + 2, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 - 1, y1), (x2 - 1, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 + 1, y1), (x2 + 1, y2), bgcolor, &blend);

            draw_antialiased_line_segment_mut(img, (x1 - 1, y1), (x2 - 1, y2), centercolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 + 1, y1), (x2 + 1, y2), centercolor, &blend);
        }

        if slope < -1.0 || 1.0 < slope {
            draw_antialiased_line_segment_mut(img, (x1, y1 + 3), (x2, y2 + 3), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 - 3), (x2, y2 - 3), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 + 2), (x2, y2 + 2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 - 2), (x2, y2 - 2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 + 1), (x2, y2 + 1), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 - 1), (x2, y2 - 1), bgcolor, &blend);

            draw_antialiased_line_segment_mut(img, (x1, y1 + 1), (x2, y2 + 1), centercolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 - 1), (x2, y2 - 1), centercolor, &blend);
        }

        if slope == 1.0 {
            draw_antialiased_line_segment_mut(img, (x1 - 3, y1), (x2 - 3, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 + 3, y1), (x2 + 3, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 - 2, y1), (x2 - 2, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 + 2, y1), (x2 + 2, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 - 1, y1), (x2 - 1, y2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 + 1, y1), (x2 + 1, y2), bgcolor, &blend);

            draw_antialiased_line_segment_mut(img, (x1 - 1, y1), (x2 - 1, y2), centercolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1 + 1, y1), (x2 + 1, y2), centercolor, &blend);
        }
        if slope == -1.0 {
            draw_antialiased_line_segment_mut(img, (x1, y1 - 3), (x2, y2 - 3), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 + 3), (x2, y2 + 3), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 - 2), (x2, y2 - 2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 + 2), (x2, y2 + 2), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 - 1), (x2, y2 - 1), bgcolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 + 1), (x2, y2 + 1), bgcolor, &blend);

            draw_antialiased_line_segment_mut(img, (x1, y1 + 1), (x2, y2 + 1), centercolor, &blend);
            draw_antialiased_line_segment_mut(img, (x1, y1 - 1), (x2, y2 - 1), centercolor, &blend);
        }

        draw_antialiased_line_segment_mut(img, s1, s2, centercolor, &blend);
        draw_antialiased_line_segment_mut(img, s1, s2, centercolor, &blend);
    }
}

struct TRFile {
    src: csv::Reader<File>,
}

impl TRFile {
    pub fn reader(file: File) -> csv::Reader<File> {
        csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b' ')
            .from_reader(file)
    }
    pub fn open(file: File) -> TRFile {
        TRFile {
            src: Self::reader(file),
        }
    }

    pub fn iter<'a>(&'a mut self) -> impl Iterator<Item = Result<TR, csv::Error>> + 'a {
        self.src.deserialize()
    }
}

type GreyBuffer = image::ImageBuffer<image::Luma<u8>, std::vec::Vec<u8>>;

fn spiral_erase(img: &mut GreyBuffer) {
    // Basic erase pattern.
    draw_fat_arc(
        img,
        TR { t: 0.0, r: 0.0 },
        TR {
            t: 100.0 * 6.28318,
            r: 1.0,
        },
        Luma([255u8]),
        Luma([128u8]),
        interpolate,
    );
}

fn render_trf(name: String, img: &mut GreyBuffer) -> Result<u32, Box<Error>> {
    println!("Opening {}", name);
    let f = File::open(name)?;
    let mut trf = TRFile::open(f);
    let mut iter = trf.iter();
    let mut last: TR = iter.next().ok_or("missing first record")??;
    let mut count = 0;

    // Draw the thing.
    for rec in iter {
        let point: TR = rec?;
        // println!("{:?} -> {:?}", last, point);
        draw_fat_arc(img, last, point, Luma([255u8]), Luma([128u8]), interpolate);
        last = point;
        count += 1;
    }
    Ok(count)
}

fn run() -> Result<(), Box<Error>> {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let mut img: GreyBuffer = GrayImage::from_pixel(1000, 1000, Luma([180u8]));
    spiral_erase(&mut img);
    let arc_count = render_trf(args[1].clone(), &mut img)?;
    img.save("out.png")?;
    println!("Rendered {} arcs to out.png", arc_count);
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
