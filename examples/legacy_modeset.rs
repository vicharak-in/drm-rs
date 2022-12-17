extern crate ascii_converter;
extern crate drm;
extern crate image;

mod utils;

use utils::*;

use drm::control::Device as ControlDevice;

use drm::buffer::DrmFourcc;

use ascii_converter::string_to_decimals;
use drm::control::{connector, crtc};

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

    // Map it and grey it out.
    {
        let args_data = std::env::args().collect::<Vec<String>>();
        let mut data: Vec<u8> = vec![
            0xEA, 0xFF, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x06, 0x00,
        ];

        // replace 11th element for data length
        data[10] = (args_data[1].len()) as u8;

        // convert string vector to decimal ascii
        let mut temp = string_to_decimals(&args_data[1]).unwrap();
        data.append(&mut temp);

        let hexdata = HexSlice::new(&data);
        println!("data: {}", hexdata);

        let mut map = card
            .map_dumb_buffer(&mut db)
            .expect("Could not map dumbbuffer");
        for b in map.as_mut() {
            for e in data.iter() {
                *b = *e;
            }
        }
    }

    // Create an FB:
    let fb = card
        .add_framebuffer(&db, 24, 24)
        .expect("Could not create FB");

    println!("{:?}", mode);
    println!("{:?}", fb);
    println!("{:?}", db);
    println!("{:?}", crtc.handle());

    // Set the crtc
    // On many setups, this requires root access.
    card.set_crtc(crtc.handle(), Some(fb), (0, 0), &[con.handle()], Some(mode))
        .expect("Could not set CRTC");

    let five_seconds = ::std::time::Duration::from_millis(2000);
    ::std::thread::sleep(five_seconds);

    card.destroy_framebuffer(fb).unwrap();
    card.destroy_dumb_buffer(db).unwrap();
}
