// Copyright (c) 2017 Brandon Thomas <bt@brand.io>, <echelon@gmail.com>
// Etherdream.rs, a library for the EtherDream laser projector DAC.
//! webcam_spiral.rs
//! This example reads in raster images captured from a webcam and uses
//! them to map colors onto a projected spiral.

extern crate camera_capture;
extern crate lase;
extern crate image;

use camera_capture::Frame;
use image::ImageBuffer;
use image::Rgb;
use lase::Point;
use lase::tools::ETHERDREAM_X_MAX as X_MAX;
use lase::tools::ETHERDREAM_X_MIN as X_MIN;
use lase::tools::find_first_etherdream_dac;
use std::f64::consts::PI;
use std::f64;
use std::sync::Arc;
use std::sync::RwLock;

/// Number of points along the spiral to sample.
static SPIRAL_POINTS : i32 = 1500;

/// Number of points to blank from spiral's edge to center.
static BLANKING_POINTS : i32 = 20;

/// Other parameters.
static SPIRAL_GROWTH : f64 = 4.0;
static MAX_RADIUS : f64 = 30000.0;

type Image = ImageBuffer<Rgb<u8>, Frame>;

fn main() {
  let frame_buffer : Arc<RwLock<Option<Image>>> = Arc::new(RwLock::new(None));
  let frame_buffer2 = frame_buffer.clone();

  let webcam_thread = std::thread::spawn(move || {
    let cam = camera_capture::create(0).expect("Could not open webcam.")
        .fps(30.0)
        .expect("Unsupported webcam fps.")
        .resolution(320, 240) // TODO: Make parameters.
        .expect("Unsupported webcam resolution.")
        .start()
        .expect("Could not begin webcam.");
    for frame in cam {
      match frame_buffer2.write() {
        Err(_) => println!("Error reading frame from webcam."),
        Ok(mut w) => {
          *w = Some(frame);
        }
      }
    }
  });

  println!("Searching for DAC...");

  let mut dac = find_first_etherdream_dac().expect("Couldn't find DAC");

  static mut pos: i32 = 0; // Position in the spiral.

  let _r = dac.play_function(|num_points: u16| {
    let mut points = Vec::new();

    for _i in 0 .. num_points {
      // Get the current point along the beam.
      // TODO: Let's build this into etherdream.rs
      // TODO: Also, let's create a `dac.play_stream(S: Stream)`.
      let f = unsafe {
        pos = (pos + 1) % (BLANKING_POINTS + SPIRAL_POINTS);
        pos
      };

      if f >= SPIRAL_POINTS {
        // Track back from end of spiral to origin.
        let (x, y) = get_blanking_point(f - SPIRAL_POINTS);
        points.push(Point::xy_binary(x, y, false));
        continue;
      }

      match frame_buffer.read() {
        Err(_) => println!("Cannot obtain read lock"),
        Ok(maybe_frame) => {
          match *maybe_frame {
            None => {
              let (x, y) = get_spiral_point(f);
              points.push(Point::xy_binary(x, y, false));
            },
            Some(ref frame) => {
              let (x, y) = get_spiral_point(f);
              let (r, g, b) = laser_color_from_webcam(&frame, x, y);
              points.push(Point::xy_rgb(x, y, r, g, b));
            },
          }
        },
      }
    }

    points
  });

  webcam_thread.join().unwrap();
}

fn get_spiral_point(cursor: i32) -> (i16, i16) {
  // FIXME: The spiral math is convoluted and unintuitive.
  let i = (cursor as f64) / SPIRAL_POINTS as f64;
  let t = i * 2.0 * PI * SPIRAL_GROWTH * 10.0;
  let r = MAX_RADIUS as f64 * i;
  // Spirals are of the form A * x * trig(x), where A is constant.
  let x = t.cos() * r;
  let y = t.sin() * r;
  (x as i16, y as i16)
}

fn get_blanking_point(cursor: i32) -> (i16, i16) {
  // Get the outermost spiral point.
  let (end_x, end_y) = get_spiral_point(SPIRAL_POINTS);
  let x = end_x as f64 - end_x as f64 * cursor as f64 / BLANKING_POINTS as f64;
  let y = end_y as f64 - end_y as f64 * cursor as f64 / BLANKING_POINTS as f64;
  (x as i16, y as i16)
}

fn laser_color_from_webcam(image: &Image,
                           x: i16, y: i16) -> (u16, u16, u16) {

  fn map_point(laser_position: i16, image_scale: u32) -> u32 {
    // NB: X_MIN and X_MAX are same values for the Y dimension.
    let num = laser_position as f64 - X_MIN as f64;
    let denom = X_MAX as f64 - X_MIN as f64;
    let ratio = num / denom;
    let scale = image_scale as f64; // "scale" is width or height.
    let result = ratio * scale;
    result as u32
  }

  fn webcam_x(laser_x: i16, image_width: u32) -> u32 {
    map_point(laser_x, image_width)
  }

  fn webcam_y(laser_y: i16, image_height : u32) -> u32 {
    let laser_y = laser_y * -1; // Inverted
    map_point(laser_y, image_height)
  }

  fn expand_color(color: u8) -> u16 {
    // Thresholding gives a sharper image / "dithering" effect.
    if color < 100 {
      0
    } else {
      (color as u16) * 257
    }
  }

  let w_x = webcam_x(x, image.width());
  let w_y = webcam_y(y, image.height());

  let pix = image.get_pixel(w_x, w_y);

  let r = expand_color(pix.data[0]);
  let g = expand_color(pix.data[1]);
  let b = expand_color(pix.data[2]);

  (r, g, b)
}

