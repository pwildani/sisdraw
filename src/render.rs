use std::mem::swap;

use image::*;
//use imageproc::drawing::draw_antialiased_line_segment_mut;

use arc::ArcToXYIter;
use tr::TR;

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
        let (x1, y1) = s1;
        let (x2, y2) = s2;
        let slope = (x2 - x1) as f32 / (y2 - y1) as f32;
        let bgc = |_| bgcolor;
        let cc = |_| centercolor;

        if -1.0 <= slope && slope < 1.0 {
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 - 3.0, y1),
                (x2 - 3.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 + 3.0, y1),
                (x2 + 3.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 - 2.0, y1),
                (x2 - 2.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 + 2.0, y1),
                (x2 + 2.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 - 1.0, y1),
                (x2 - 1.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 + 1.0, y1),
                (x2 + 1.0, y2),
                bgc,
                &blend,
            );

            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 - 1.0, y1),
                (x2 - 1.0, y2),
                cc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 + 1.0, y1),
                (x2 + 1.0, y2),
                cc,
                &blend,
            );
        }

        if slope < -1.0 || 1.0 < slope {
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 + 3.0),
                (x2, y2 + 3.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 - 3.0),
                (x2, y2 - 3.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 + 2.0),
                (x2, y2 + 2.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 - 2.0),
                (x2, y2 - 2.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 + 1.0),
                (x2, y2 + 1.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 - 1.0),
                (x2, y2 - 1.0),
                bgc,
                &blend,
            );

            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 + 1.0),
                (x2, y2 + 1.0),
                cc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 - 1.0),
                (x2, y2 - 1.0),
                cc,
                &blend,
            );
        }

        if slope == 1.0 {
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 - 3.0, y1),
                (x2 - 3.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 + 3.0, y1),
                (x2 + 3.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 - 2.0, y1),
                (x2 - 2.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 + 2.0, y1),
                (x2 + 2.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 - 1.0, y1),
                (x2 - 1.0, y2),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 + 1.0, y1),
                (x2 + 1.0, y2),
                bgc,
                &blend,
            );

            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 - 1.0, y1),
                (x2 - 1.0, y2),
                cc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1 + 1.0, y1),
                (x2 + 1.0, y2),
                cc,
                &blend,
            );
        }
        if slope == -1.0 {
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 - 3.0),
                (x2, y2 - 3.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 + 3.0),
                (x2, y2 + 3.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 - 2.0),
                (x2, y2 - 2.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 + 2.0),
                (x2, y2 + 2.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 - 1.0),
                (x2, y2 - 1.0),
                bgc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 + 1.0),
                (x2, y2 + 1.0),
                bgc,
                &blend,
            );

            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 + 1.0),
                (x2, y2 + 1.0),
                cc,
                &blend,
            );
            draw_gradient_antialiased_line_segment_mut(
                img,
                (x1, y1 - 1.0),
                (x2, y2 - 1.0),
                cc,
                &blend,
            );
        }

        draw_gradient_antialiased_line_segment_mut(img, s1, s2, cc, &blend);
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
