extern crate csv;
extern crate image;
extern crate imageproc;
extern crate serde;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

use std::mem::swap;
use std::cmp::max;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

use clap::{App, Arg};

use image::*;
//use imageproc::drawing::draw_antialiased_line_segment_mut;
use imageproc::pixelops::interpolate;

arg_enum!{
    #[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
    enum ErasePattern {
        NONE,
        SPIRAL,
    }
}

impl Default for ErasePattern {
    fn default() -> ErasePattern {
        ErasePattern::NONE
    }
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct Config {
    erase: ErasePattern,
    geometry: String, // struct {width: u32, height: u32},
    thr_filename: OsString,
    out_filename: OsString,
    undrawn_color: u8,
    sand_color: u8,
    table_color: u8,
}

impl Config {
    pub fn geo_dims(&self) -> (u32, u32) {
        (1000, 1000)
    }
}

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
    fn on(&self, dims: (u32, u32)) -> (f32, f32) {
        let (w, h) = dims;
        let w2 = w as f64 / 2.0;
        let h2 = h as f64 / 2.0;
        ((w2 + self.x * w2) as f32, (h2 + self.y * h2) as f32)
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
        let bgc = |_| bgcolor;
        let cc = |_| centercolor;

        if -1.0 <= slope && slope < 1.0 {
            draw_gradient_antialiased_line_segment_mut(img, (x1 - 3.0, y1), (x2 - 3.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 + 3.0, y1), (x2 + 3.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 - 2.0, y1), (x2 - 2.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 + 2.0, y1), (x2 + 2.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 - 1.0, y1), (x2 - 1.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 + 1.0, y1), (x2 + 1.0, y2), bgc, &blend);

            draw_gradient_antialiased_line_segment_mut(img, (x1 - 1.0, y1), (x2 - 1.0, y2), cc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 + 1.0, y1), (x2 + 1.0, y2), cc, &blend);
        }

        if slope < -1.0 || 1.0 < slope {
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 + 3.0), (x2, y2 + 3.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 - 3.0), (x2, y2 - 3.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 + 2.0), (x2, y2 + 2.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 - 2.0), (x2, y2 - 2.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 + 1.0), (x2, y2 + 1.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 - 1.0), (x2, y2 - 1.0), bgc, &blend);

            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 + 1.0), (x2, y2 + 1.0), cc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 - 1.0), (x2, y2 - 1.0), cc, &blend);
        }

        if slope == 1.0 {
            draw_gradient_antialiased_line_segment_mut(img, (x1 - 3.0, y1), (x2 - 3.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 + 3.0, y1), (x2 + 3.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 - 2.0, y1), (x2 - 2.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 + 2.0, y1), (x2 + 2.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 - 1.0, y1), (x2 - 1.0, y2), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 + 1.0, y1), (x2 + 1.0, y2), bgc, &blend);

            draw_gradient_antialiased_line_segment_mut(img, (x1 - 1.0, y1), (x2 - 1.0, y2), cc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1 + 1.0, y1), (x2 + 1.0, y2), cc, &blend);
        }
        if slope == -1.0 {
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 - 3.0), (x2, y2 - 3.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 + 3.0), (x2, y2 + 3.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 - 2.0), (x2, y2 - 2.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 + 2.0), (x2, y2 + 2.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 - 1.0), (x2, y2 - 1.0), bgc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 + 1.0), (x2, y2 + 1.0), bgc, &blend);

            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 + 1.0), (x2, y2 + 1.0), cc, &blend);
            draw_gradient_antialiased_line_segment_mut(img, (x1, y1 - 1.0), (x2, y2 - 1.0), cc, &blend);
        }

        draw_gradient_antialiased_line_segment_mut(img, s1, s2, cc, &blend);
        draw_gradient_antialiased_line_segment_mut(img, s1, s2, cc, &blend);
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

fn spiral_erase(config: &Config, img: &mut GreyBuffer) {
    // Basic erase pattern.
    draw_fat_arc(
        img,
        TR { t: 0.0, r: 0.0 },
        TR {
            t: 100.0 * 6.28318,
            r: 1.0,
        },
        Luma([config.sand_color]),
        Luma([config.table_color]),
        interpolate,
    );
}

fn render_trf(config: &Config, name: OsString, img: &mut GreyBuffer) -> Result<u32, Box<Error>> {
    println!("Opening {}", name.to_string_lossy());
    let f = File::open(name)?;
    let mut trf = TRFile::open(f);
    let mut iter = trf.iter();
    let mut last: TR = iter.next().ok_or("missing first record")??;
    let mut count = 0;

    // Draw the thing.
    for rec in iter {
        let point: TR = rec?;
        // println!("{:?} -> {:?}", last, point);
        draw_fat_arc(
            img,
            last,
            point,
            Luma([config.sand_color]),
            Luma([config.table_color]),
            interpolate,
        );
        last = point;
        count += 1;
    }
    Ok(count)
}

fn run(config: Config) -> Result<(), Box<Error>> {
    println!("Hello, world!");
    let dims = config.geo_dims();
    let mut img: GreyBuffer = GrayImage::from_pixel(dims.0, dims.1, Luma([config.undrawn_color]));
    match config.erase {
        ErasePattern::NONE => (),
        ErasePattern::SPIRAL => spiral_erase(&config, &mut img),
    }

    let arc_count = render_trf(&config, config.thr_filename.clone(), &mut img)?;
    img.save(config.out_filename.clone())?;
    println!(
        "Rendered {} arcs to {}",
        arc_count,
        config.out_filename.to_string_lossy()
    );
    Ok(())
}

fn main() {
    let args = App::new("sis_draw")
        .about("Render a .thr file")
        .arg(
            Arg::with_name("erase")
                .short("E")
                .long("erase")
                .takes_value(true)
                .possible_values(&ErasePattern::variants())
                .default_value("none")
                .case_insensitive(true)
                .value_name("PATTERN")
                .help("Render an erase pattern beforehand"),
        )
        /*
        .arg(Arg::with_name("geometry")
             .default_value("1000x1000")
             .takes_value(true)
             .value_name("GEOMETRY")
             .help("The dimensions of the output image"))
         */
        .arg(
            Arg::with_name("thr_filename")
                .value_name("THR_FILE")
                .index(2)
                // .multiple
                .required(true)
                .help("The pattern to draw"),
        ).arg(
            Arg::with_name("out_filename")
                .index(1)
                .short("o")
                .long("out")
                .value_name("IMAGE")
                .required(true)
                .help("Where to write the image. Format is derived from the extension"),
        ).get_matches();

    let config = Config {
        erase: value_t!(args, "erase", ErasePattern).unwrap(),
        geometry: "1000x1000".to_string(),
        thr_filename: args.value_of_os("thr_filename").unwrap().to_os_string(),
        out_filename: args.value_of_os("out_filename").unwrap().to_os_string(),
        undrawn_color: 220,
        sand_color: 255,
        table_color: 180,
    };

    if let Err(err) = run(config) {
        println!("{}", err);
        process::exit(1);
    }
}


pub fn draw_gradient_antialiased_line_segment_mut<I, B, C>(
    image: &mut I,
    start: (f32, f32),
    end: (f32, f32),
    color: C,
    blend: B,
) where
    I: GenericImage,
    I::Pixel: 'static,
    B: Fn(I::Pixel, I::Pixel, f32) -> I::Pixel,
    C: Fn(f32) -> I::Pixel,
{
    let (mut x0, mut y0) = (start.0, start.1);
    let (mut x1, mut y1) = (end.0, end.1);

    let is_steep = (y1 - y0).abs() > (x1 - x0).abs();

    if is_steep {
        if y0 > y1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }
        let plotter = Plotter {
            image: image,
            transform: |x, y| (y, x),
            blend: blend,
        };
        plot_wu_line(plotter, (y0, x0), (y1, x1), color);
    } else {
        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }
        let plotter = Plotter {
            image: image,
            transform: |x, y| (x, y),
            blend: blend,
        };
        plot_wu_line(plotter, (x0, y0), (x1, y1), color);
    };
}

fn plot_wu_line<I, T, B, C>(
    mut plotter: Plotter<I, T, B>,
    start: (f32, f32),
    end: (f32, f32),
    color: C,
) where
    I: GenericImage,
    I::Pixel: 'static,
    T: Fn(f32, f32) -> (f32, f32),
    C: Fn(f32) -> I::Pixel,
    B: Fn(I::Pixel, I::Pixel, f32) -> I::Pixel,
{
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let gradient = dy as f32 / dx as f32;
    let mut fy = start.1 as f32;

    let offset = start.0 - (start.0 as u32) as f32;
    for xi in (start.0 as u32) ..(end.0 as u32 + 1) {
        let x = xi as f32 + offset;
        let frac = x as f32 / (start.0 - end.0 + 1.0) as f32;
        let c = color(frac);
        plotter.plot(x, fy as f32, c, 1.0 - fy.fract());
        plotter.plot(x, fy as f32 + 1.0, c, fy.fract());
        fy += gradient;
    }
}

struct Plotter<'a, I: 'a, T, B>
where
    I: GenericImage,
    I::Pixel: 'static,
    T: Fn(f32, f32) -> (f32, f32),
    B: Fn(I::Pixel, I::Pixel, f32) -> I::Pixel,
{
    image: &'a mut I,
    transform: T,
    blend: B,
}

impl<'a, I, T, B> Plotter<'a, I, T, B>
where
    I: GenericImage,
    I::Pixel: 'static,
    T: Fn(f32, f32) -> (f32, f32),
    B: Fn(I::Pixel, I::Pixel, f32) -> I::Pixel,
{
    fn in_bounds(&self, x: f32, y: f32) -> bool {
        x >= 0.0 && x < self.image.width() as f32 && y >= 0.0 && y < self.image.height() as f32
    }

    pub fn plot(&mut self, x: f32, y: f32, line_color: I::Pixel, line_weight: f32) {
        let (x_trans, y_trans) = (self.transform)(x, y);
        if self.in_bounds(x_trans, y_trans) {
            let original = self.image.get_pixel(x_trans as u32, y_trans as u32);
            let blended = (self.blend)(line_color, original, line_weight);
            self.image.put_pixel(
                x_trans as u32,
                y_trans as u32,
                blended,
            );
        }
    }
}
