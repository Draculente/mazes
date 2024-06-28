use std::{
    cmp::Reverse,
    collections::HashMap,
    fmt::{Display, Write},
    sync::Arc,
};

use anyhow::anyhow;
use image::Rgba;
use itertools::Itertools;
use priority_queue::PriorityQueue;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
enum BlockType {
    White,
    Black,
    Orange,
    Blue,
    Green,
    Yellow,
    Border,
}

impl BlockType {
    fn from_rgba(rgba: &Rgba<u8>) -> Self {
        let red = *rgba.0.get(0).expect("Rgba needs to have red");
        let green = *rgba.0.get(1).expect("Rgba needs to have green");
        let blue = *rgba.0.get(2).expect("Rgba needs to have blue");

        return match (red, green, blue) {
            (0, 0, 0) => BlockType::Black,
            (255, 255, 255) => BlockType::White,
            (200, 113, 55) => BlockType::Orange,
            (255, 255, 0) => BlockType::Yellow,
            (0, 255, 0) => BlockType::Green,
            (0, 0, 255) => BlockType::Blue,
            _ => BlockType::Border,
        };
    }

    fn is_border(&self) -> bool {
        *self == BlockType::Border
    }
}

impl Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BlockType::White => "⬜",
            BlockType::Black => "⬛",
            BlockType::Orange => "🟧",
            BlockType::Blue => "🟦",
            BlockType::Green => "🟩",
            BlockType::Yellow => "🟨",
            BlockType::Border => "🟥",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Block {
    x: u16,
    y: u16,
    block_type: BlockType,
}

impl Block {
    fn new(x: u16, y: u16, block_type: BlockType) -> Self {
        Self { x, y, block_type }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.block_type.to_string().as_str())
    }
}

struct Map {
    width: u16,
    height: u16,
    blocks: Vec<Vec<Block>>,
}

impl Map {
    fn new(blocks: Vec<Vec<Block>>) -> Self {
        let width = blocks
            .get(0)
            .expect("A map must at least have a height of 1")
            .len() as u16;
        let height = blocks.len() as u16;

        Self {
            width,
            height,
            blocks,
        }
    }

    fn get_block(&self, x: u16, y: u16) -> Option<Block> {
        let x_usize = x as usize;
        let y_usize = y as usize;
        self.blocks
            .get(y_usize)
            .and_then(|row: &Vec<Block>| row.get(x_usize).cloned())
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.blocks {
            for block in row {
                f.write_str(block.to_string().as_str())?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

fn is_border_row(row: &Vec<&mut Rgba<u8>>) -> bool {
    // row.all(|&p| BlockType::from_rgba(&p).is_border())
    row.iter()
        .map(|rgba| BlockType::from_rgba(rgba))
        .all(|block| block.is_border())
}

fn get_blocks_from_pixel_row(block_row_y: usize, pixel_row: &Vec<&mut Rgba<u8>>) -> Vec<Block> {
    pixel_row
        .split(|rgba| BlockType::from_rgba(rgba).is_border())
        .filter(|pixel_block| pixel_block.len() > 2)
        .map(|pixel_block| {
            // Get the third pixel of the block to get pure color (on the edges of each block are blurred colors due to compression)
            pixel_block
                .get(2)
                .expect("Each block must have at least a width of 3 pixels")
        })
        .enumerate()
        .map(|(block_x, rgba)| {
            Block::new(
                block_x as u16,
                block_row_y as u16,
                BlockType::from_rgba(rgba),
            )
        })
        .collect_vec()
}

fn main() {
    let mut img = image::open("./images/lageplan.png").expect("Error opening the image");
    let rgba8_img = img
        .as_mut_rgba8()
        .expect("Failed to convert image to rgba8.");

    // Every pixel row of a chunk belongs to the same block.
    let row_chunks = rgba8_img
        .enumerate_rows_mut()
        .map(|(_, row)| row.map(|(_, _, rgba)| rgba).collect_vec())
        .chunk_by(is_border_row);

    let blocks = row_chunks
        .into_iter()
        .filter(|(is_border_row, _)| !is_border_row)
        .map(|(_, chunk)| {
            // Take the third pixel row to get pure colors (on the edges of each block are "blurred pixels" due to compression)
            chunk
                .skip(2)
                .take(1)
                .nth(0)
                .expect("Each row must at least have a height of 5 pixels")
        })
        .enumerate()
        .map(|(block_row_y, row)| get_blocks_from_pixel_row(block_row_y, &row))
        .collect_vec();

    let map = Map::new(blocks);

    print!("{map}");
}
