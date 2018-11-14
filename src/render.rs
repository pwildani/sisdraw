use std::mem::swap;

use image::*;
//use imageproc::drawing::draw_antialiased_line_segment_mut;

use arc::ArcToXYIter;
use tr::TR;

#[derive(Debug, Clone)]
struct Point(pub f32, pub f32);

impl Point {
    fn plus(&self, delta: (f32, f32)) -> (f32, f32) {
        (self.0 + delta.0, self.1 + delta.1)
    }

    fn times(&self, scalar: f32) -> (f32, f32) {
        (self.0 * scalar, self.1 * scalar)
    }
}

impl From<(f32, f32)> for Point {
    fn from(a: (f32, f32)) -> Point {
        Point(a.0, a.1)
    }
}
impl Into<(f32, f32)> for Point {
    fn into(self) -> (f32, f32) {
        (self.0, self.1)
    }
}

fn draw_wide<I, P, B, C>(halfwidth: i32, img: &mut I, start: P, end: P, color: C, blend: B)
where
    I: GenericImage,
    I::Pixel: 'static,
    P: Into<Point>,
    B: Fn(I::Pixel, I::Pixel, f32) -> I::Pixel,
    C: Fn(f32) -> I::Pixel,
{
    let s1 = start.into();
    let s2 = end.into();
    let (x1, y1) = s1.clone().into();
    let (x2, y2) = s2.clone().into();
    let delta: Point;
    if y1 == y2 {
        delta = Point(0.0, 1.0);
    } else if x1 == x2 {
        delta = Point(1.0, 0.0);
    } else {
        // One unit step perpendicular to the line.
        let slope = (x2 - x1) as f32 / (y2 - y1) as f32;
        let invlen = (slope * slope + 1.0).sqrt().recip();
        // Normalize [slope, 1] to unit length.
        let dx = slope * invlen;
        let dy = 1.0 * invlen;
        // Rotate 90 degrees. (== [[dx, dy]] * [[cos(90), -sin(90)][sin(90), cos(90)]])
        let rx = dy;
        let ry = -dx;
        delta = Point(rx, ry);
    }
    for i in -halfwidth..=halfwidth {
        let d = delta.times(i as f32);
        //println!("Delta: {:?} * i: {:?} -> d {:?}", delta, i, d);
        draw_gradient_antialiased_line_segment_mut(img, s1.plus(d), s2.plus(d), &color, &blend);
    }
}

pub fn draw_fat_arc<I, B>(
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
        let bgc = |_| bgcolor;
        draw_wide(3, img, s1, s2, bgc, &blend);

        let cc = |_| centercolor;
        draw_wide(1, img, s1, s2, cc, &blend);
        draw_gradient_antialiased_line_segment_mut(img, s1, s2, cc, &blend);
    }
}

pub type GreyBuffer = image::ImageBuffer<image::Luma<u8>, std::vec::Vec<u8>>;

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
    for xi in (start.0 as u32)..(end.0 as u32 + 1) {
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
            self.image
                .put_pixel(x_trans as u32, y_trans as u32, blended);
        }
    }
}
