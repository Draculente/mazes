use std::fmt::Display;

use image::{DynamicImage, Rgba};
use itertools::Itertools;

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
        let red = *rgba.0.first().expect("Rgba needs to have red");
        let green = *rgba.0.first().expect("Rgba needs to have green");
        let blue = *rgba.0.first().expect("Rgba needs to have blue");

        match (red, green, blue) {
            (0, 0, 0) => BlockType::Black,
            (255, 255, 255) => BlockType::White,
            (200, 113, 55) => BlockType::Orange,
            (255, 255, 0) => BlockType::Yellow,
            (0, 255, 0) => BlockType::Green,
            (0, 0, 255) => BlockType::Blue,
            _ => BlockType::Border,
        }
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
pub struct Block {
    pub x: usize,
    pub y: usize,
    block_type: BlockType,
}

impl Block {
    fn new(x: usize, y: usize, block_type: BlockType) -> Self {
        Self { x, y, block_type }
    }

    pub fn is_walkable(&self) -> bool {
        !(self.block_type == BlockType::Black || self.block_type == BlockType::White)
    }

    pub fn speed(&self) -> usize {
        match self.block_type {
            BlockType::White => usize::MAX,
            BlockType::Black => usize::MAX,
            BlockType::Orange => 5,
            BlockType::Blue => 2,
            BlockType::Green => 1,
            BlockType::Yellow => 7,
            BlockType::Border => usize::MAX,
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.block_type.to_string().as_str())
    }
}

pub struct Map {
    width: usize,
    height: usize,
    blocks: Vec<Vec<Block>>,
}

impl Map {
    pub fn new(blocks: Vec<Vec<Block>>) -> Self {
        let width = blocks
            .first()
            .expect("A map must at least have a height of 1")
            .len() as usize;
        let height = blocks.len() as usize;

        Self {
            width,
            height,
            blocks,
        }
    }

    pub fn get_block(&self, x: usize, y: usize) -> Option<Block> {
        self.blocks
            .get(y)
            .and_then(|row: &Vec<Block>| row.get(x).cloned())
    }

    pub fn get_reachable(&self, x: usize, y: usize) -> Vec<Block> {
        let mut reachable_blocks = vec![];

        // To the left
        if x > 0 {
            reachable_blocks.push(self.get_block(x - 1, y));
        }
        // To the top
        if y > 0 {
            reachable_blocks.push(self.get_block(x, y - 1));
        }
        // To the right
        if x + 1 <= self.width {
            reachable_blocks.push(self.get_block(x + 1, y));
        }
        // To the bottom
        if y + 1 <= self.height {
            reachable_blocks.push(self.get_block(x, y + 1));
        }

        reachable_blocks
            .into_iter()
            .filter(|b| b.is_some())
            .collect::<Option<Vec<_>>>()
            .expect("Reachable blocks should not be empty")
            .into_iter()
            .filter(|b| b.is_walkable())
            .collect_vec()
    }

    pub fn to_string_with_location(&self, location: Option<Block>) -> String {
        let mut res = "".to_string();
        for row in &self.blocks {
            for block in row {
                if let Some(inserted_block) = location {
                    if block.x == inserted_block.x && block.y == inserted_block.y {
                        res += "🤖";
                        continue;
                    }
                }
                res += &block.to_string();
            }
            res += "\n";
        }
        res
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string_with_location(None))?;
        Ok(())
    }
}

impl From<DynamicImage> for Map {
    fn from(mut img: DynamicImage) -> Self {
        // let mut img = image::open("./images/lageplan.png").expect("Error opening the image");
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

        Map::new(blocks)
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
                block_x as usize,
                block_row_y as usize,
                BlockType::from_rgba(rgba),
            )
        })
        .collect_vec()
}
