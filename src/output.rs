use crate::automaton::{TiledGrid, TILE_SIZE};
use crate::{Automaton, TiledAutomaton};
use gif::{Encoder, Frame};
use std::fs::File;

pub fn write_to_gif_file(
    fname: Option<&str>,
    autom: &mut Automaton,
    scale: u16,
    steps: u32,
    skip: u32,
) {
    let size = autom.size as u16;
    let scaled_size = size * scale;
    let states = autom.states;

    let mut im_file = File::create(fname.unwrap_or("test.gif")).unwrap();
    let mut g = Encoder::new(&mut im_file, scaled_size, scaled_size, &[]).unwrap();
    g.set_repeat(gif::Repeat::Infinite).unwrap();

    let u = autom.skipped_iter(steps, skip);
    let mut c = 0;
    let palette = make_palette(states);
    let frames = u.map(|s| {
        let inf = Frame::from_palette_pixels(
            scaled_size,
            scaled_size,
            &duplicate_array(s, size as usize, scale),
            &palette,
            None,
        );
        println!("Processing image {}", c);
        c += 1;
        inf
    });
    for frame in frames {
        g.write_frame(&frame).expect("Error writing frame");
    }
}

pub fn write_to_gif_file_tiled(
    fname: Option<&str>,
    autom: &mut TiledAutomaton,
    scale: u16,
    steps: u32,
    skip: u32,
) {
    let size = autom.size as u16;
    let scaled_size = size * scale;
    let states = autom.states;

    let mut im_file = File::create(fname.unwrap_or("test.gif")).unwrap();
    let mut g = Encoder::new(&mut im_file, scaled_size, scaled_size, &[]).unwrap();
    g.set_repeat(gif::Repeat::Infinite).unwrap();

    let u = autom.skipped_iter(steps, skip);
    let mut c = 0;
    let palette = make_palette(states);
    let frames = u.map(|s| {
        let inf = Frame::from_palette_pixels(
            scaled_size,
            scaled_size,
            &duplicate_array_tiled(s, size as usize, scale),
            &palette,
            None,
        );
        println!("Processing image {}", c);
        c += 1;
        inf
    });
    for frame in frames {
        g.write_frame(&frame).expect("Error writing frame");
    }
}

fn make_palette(states: u8) -> Vec<u8> {
    let col_1 = [255., 255., 255.];
    let col_2 = [0., 0., 255.];

    let mut palette = vec![];
    for x in 0..states {
        let t = (x as f64) / states as f64;
        palette.push((col_1[0] * t + col_2[0] * (1. - t)) as u8);
        palette.push((col_1[1] * t + col_2[1] * (1. - t)) as u8);
        palette.push((col_1[2] * t + col_2[2] * (1. - t)) as u8);
    }
    palette
}

#[inline]
fn duplicate_array(s: Vec<u8>, size: usize, scale: u16) -> Vec<u8> {
    let scaled_size = size * scale as usize;
    let mut out = Vec::with_capacity(scaled_size * scaled_size);
    for i in 0..scaled_size {
        for j in 0..scaled_size {
            let item = s[(i / scale as usize) * size + (j / scale as usize)];
            out.push(item);
        }
    }
    out
}

#[inline]
fn duplicate_array_tiled(s: TiledGrid, size: usize, scale: u16) -> Vec<u8> {
    let scaled_size = size * scale as usize;
    let n_tiles = size / TILE_SIZE;
    let mut out = Vec::with_capacity(scaled_size * scaled_size);
    for a in 0..scaled_size {
        for b in 0..scaled_size {
            let i = a / scale as usize;
            let j = b / scale as usize;
            let tx = i / (TILE_SIZE - 1);
            let ty = j / (TILE_SIZE - 1);
            let x = i % (TILE_SIZE - 1);
            let y = j % (TILE_SIZE - 1);
            let item = s[tx * n_tiles + ty][x * TILE_SIZE + y];
            out.push(item);
        }
    }
    out
}
