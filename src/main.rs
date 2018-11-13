extern crate csv;
extern crate image;
extern crate imageproc;
extern crate serde;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

mod arc;
mod render;
mod tr;
mod trfile;
mod xy;

use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;

use clap::{App, Arg};

use image::*;
//use imageproc::drawing::draw_antialiased_line_segment_mut;
use imageproc::pixelops::interpolate;

use render::draw_fat_arc;
use render::GreyBuffer;
use tr::TR;
use trfile::TRFile;

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

/// Renderer configuration.
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
        table_color: 140,
    };

    if let Err(err) = run(config) {
        println!("{}", err);
        process::exit(1);
    }
}
