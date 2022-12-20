extern crate ascii_converter;
extern crate clap;
extern crate drm;
extern crate image;

mod utils;
use utils::*;

use drm::buffer::DrmFourcc;
use drm::control::Device as ControlDevice;

use ascii_converter::string_to_decimals;
use clap::Parser;
use drm::control::{connector, crtc};
use std::time::Instant;

#[allow(dead_code)]
struct HexSlice<'a>(&'a [u8]);

impl<'a> HexSlice<'a> {
    fn new<T>(data: &'a T) -> HexSlice<'a>
    where
        T: ?Sized + AsRef<[u8]> + 'a,
    {
        HexSlice(data.as_ref())
    }
}

impl std::fmt::Display for HexSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.0 {
            write!(f, "{:#04X} ", b)?;
        }

        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct FrameData {
    data: Vec<String>,
}

pub fn main() {
    let card = Card::open_global();

    // Load the information.
    let res = card
        .resource_handles()
        .expect("Could not load normal resource ids.");

    let coninfo: Vec<connector::Info> = res
        .connectors()
        .iter()
        .flat_map(|con| card.get_connector(*con, true))
        .collect();

    let crtcinfo: Vec<crtc::Info> = res
        .crtcs()
        .iter()
        .flat_map(|crtc| card.get_crtc(*crtc))
        .collect();

    // Filter each connector until we find one that's connected.
    let con = coninfo
        .iter()
        .find(|&i| i.state() == connector::State::Connected)
        .expect("No connected connectors");

    // Get the first (usually best) mode
    let &mode = con.modes().get(0).expect("No modes found on connector");

    let (disp_width, disp_height) = mode.size();

    // Find a crtc and FB
    let crtc = crtcinfo.get(0).expect("No crtcs found");

    // Select the pixel format
    let fmt = DrmFourcc::Rgb888;

    // Create a DB
    // If buffer resolution is larger than display resolution, an ENOSPC (not enough video memory)
    // error may occur
    let mut db = card
        .create_dumb_buffer((disp_width.into(), disp_height.into()), fmt, 24)
        .expect("Could not create dumb buffer");

    {
        // Get data as argument from environment
        let args_data = FrameData::parse();

        // packet data from start of frame until data len
        let mut data: Vec<u8> = vec![
            0xEA, 0xFF, 0x99, 0x88, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        // Replace 11th element for data length
        data[10] = (args_data.data[0].len()) as u8;

        let mut temp = string_to_decimals(&args_data.data[0]).expect("string_to_decimals failed!");
        data.append(&mut temp);

        let dsize = 800 * 600 * 3;
        let mut i = 0usize;

        let mut map = card
            .map_dumb_buffer(&mut db)
            .expect("Could not map dumbbuffer");

        let start = Instant::now();
        for b in map.as_mut() {
            while i < dsize {
                if i < (data[10usize] + 12).into() {
                    //println!("i: {}, data: {} ", i, data[i]);
                    *b = data[i];
                } else {
                    //println!("i: {} = 0", i);
                    *b = 0 as u8;
                }

                i = i + 1;
            }
        }
        let duration = start.elapsed();
        println!("Time elapsed for buffer: {:?}", duration);
    }

    // Create an FB:
    let fb = card
        .add_framebuffer(&db, 24, 24)
        .expect("Could not create FB");

    /*
     *    println!("{:?}", mode);
     *    println!("{:?}", fb);
     *    println!("{:?}", db);
     *    println!("{:?}", crtc.handle());
     *
     */

    // Set the crtc
    // On many setups, this requires root access.
    card.set_crtc(crtc.handle(), Some(fb), (0, 0), &[con.handle()], Some(mode))
        .expect("Could not set CRTC");

    //    let five_seconds = ::std::time::Duration::from_millis(50);
    //    ::std::thread::sleep(five_seconds);

    card.destroy_framebuffer(fb).unwrap();
    card.destroy_dumb_buffer(db).unwrap();
}
