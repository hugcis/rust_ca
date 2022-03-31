//! The output utilities. Use to save the CA state to an output GIF.

use crate::automaton::AutomatonImpl;
use gif::{Encoder, Frame};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Write the CA state to a GIF file.
pub fn write_to_gif_file<P: AsRef<Path>, T>(
    path: Option<P>,
    autom: &mut T,
    scale: u16,
    steps: u32,
    skip: u32,
    delay: u16,
    rotate: u8,
) -> Result<(), io::Error>
where
    T: AutomatonImpl,
{
    let size = autom.size() as u16;
    let scaled_size = size * scale;
    let states = autom.states();

    let mut im_file = if let Some(path) = path {
        Box::new(File::create(path)?) as Box<dyn Write>
    } else {
        Box::new(io::stdout()) as Box<dyn Write>
    };

    let mut g = Encoder::new(&mut im_file, scaled_size, scaled_size, &[]).unwrap();
    g.set_repeat(gif::Repeat::Infinite).unwrap();

    let autom_iterator = autom.skipped_iter(steps, skip, scale);
    let mut c = 0;
    let palette = make_palette(states, rotate);
    let frames = autom_iterator.map(|grid| {
        let mut frame = Frame::from_palette_pixels(scaled_size, scaled_size, &grid, &palette, None);
        frame.delay = delay;
        eprint!("\rProcessing image {}/{}", c + 1, steps / skip);
        c += 1;
        frame
    });
    for frame in frames {
        g.write_frame(&frame).expect("Error writing frame");
    }
    eprintln!();
    Ok(())
}

fn make_palette(states: u8, rotate: u8) -> Vec<u8> {
    let col_1 = [255., 255., 255.];
    let col_2 = [0., 0., 255.];

    let mut palette = vec![];
    for x in 0..states {
        let t = (((x + rotate) % states) as f64) / ((states - 1) as f64);
        palette.push((col_1[0] * t + col_2[0] * (1. - t)) as u8);
        palette.push((col_1[1] * t + col_2[1] * (1. - t)) as u8);
        palette.push((col_1[2] * t + col_2[2] * (1. - t)) as u8);
    }
    palette
}
