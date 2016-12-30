extern crate laser;

use laser::etherdream;
use laser::ilda;
use laser::ilda::animation::Animation;
use laser::etherdream::dac::Dac;
use laser::etherdream::protocol::Point;

fn main() {
  println!("Reading ILDA file...");

  let filename = "./ild/datboi.ild"; // Works
  let filename = "./ild/cogz99.ild"; // Works (animated)
  let filename = "./ild/font_impact.ild"; // Works
  let filename = "./ild/font_lucida.ild"; // Works
  let filename = "./ild/thunda2.ild"; // Works (animated)
  let filename = "./ild/nyancat.ild"; // Works!! :D
  //let filename = "./ild/koolaidman.ild"; // TODO: Doesn't render correctly?

  let animation = match Animation::read_file(filename) {
    Ok(animation) => animation,
    Err(e) => {
      println!("Error: {:?}", e);
      panic!();
    },
  };

  println!("Animation Len: {}", &animation.frame_count());

  println!("Searching for EtherDream DAC...");
  let search_result = etherdream::network::find_first_dac().unwrap();

  let ip_address = search_result.ip_address;
  println!("Found dac: {}", ip_address);

  let mut dac = Dac::new(ip_address);

  let mut frame_index = 0;
  let mut frame_repeat_count = 0;
  let mut point_index = 0;

  // TODO: Draw something interesting.
  dac.play_function(move |num_points: u16| {
    let limit = num_points as usize;
    let mut buf = Vec::new();

    while buf.len() < limit {
      match animation.get_frame(frame_index) {
        None => {
          frame_index = 0;
          point_index = 0;
          continue;
        },
        Some(ref frame) => {
          match frame.get_point(point_index) {
            None => {
              // NB: Repeat slows the animation speed.
              frame_repeat_count += 1;
              if frame_repeat_count > 2 {
                frame_index += 1;
                frame_repeat_count = 0;
              }
              point_index = 0;
              continue;
            },
            Some(ref point) => {
              let r = color(point.r);
              let g = color(point.g);
              let b = color(point.b);
              buf.push(Point::xy_rgb(point.x, point.y, r, g, b));
              //buf.push(Point::xy_rgb(point.x, point.y, point.r, point.g, point.b));
              point_index += 1;
            }
          }
        },
      }
    }

    buf
  });
}

/// Map a single channel of the ILDA color range to the EtherDream color range.
fn color(color: u8) -> u16 {
  (color as u16) << 8
}
