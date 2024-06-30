use std::fmt::Display;

use image::{DynamicImage, Rgba, RgbaImage};
use itertools::Itertools;

use crate::maze_generation::{Cell, Color, MazeMap, Wall};

const IMAGE_BORDER_WIDTH: usize = 3;
const IMAGE_BLOCK_WIDTH: usize = 20;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
enum BlockType {
    White,
    Black,
    Orange,
    Blue,
    Green,
    Yellow,
    Border,
    Solution,
}

impl BlockType {
    fn from_rgba(rgba: &Rgba<u8>) -> Self {
        let red = *rgba.0.first().expect("Rgba needs to have red");
        let green = *rgba.0.get(1).expect("Rgba needs to have green");
        let blue = *rgba.0.get(2).expect("Rgba needs to have blue");

        match (red, green, blue) {
            (0, 0, 0) => BlockType::Black,
            (255, 255, 255) => BlockType::White,
            (200, 113, 55) => BlockType::Orange,
            (255, 255, 0) => BlockType::Yellow,
            (0, 255, 0) => BlockType::Green,
            (0, 0, 255) => BlockType::Blue,
            (138, 74, 243) => BlockType::Solution,
            _ => BlockType::Border,
        }
    }

    fn to_rgba(&self) -> [u8; 4] {
        match self {
            BlockType::White => [255, 255, 255, 0],
            BlockType::Black => [0, 0, 0, 255],
            BlockType::Orange => [200, 113, 55, 255],
            BlockType::Blue => [0, 0, 255, 255],
            BlockType::Green => [0, 255, 0, 255],
            BlockType::Yellow => [255, 255, 0, 255],
            BlockType::Border => [255, 0, 0, 255],
            BlockType::Solution => [138, 74, 243, 255],
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
            BlockType::Solution => "🤖",
        };
        f.write_str(s)
    }
}

impl From<Color> for BlockType {
    fn from(value: Color) -> Self {
        match value {
            Color::Blue => BlockType::Blue,
            Color::Orange => BlockType::Orange,
            Color::Yellow => BlockType::Yellow,
            Color::Green => BlockType::Green,
        }
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

    /// The smaller the better!!!
    pub fn speed(&self) -> usize {
        match self.block_type {
            BlockType::White => usize::MAX,
            BlockType::Black => usize::MAX,
            BlockType::Orange => 5,
            BlockType::Blue => 2,
            BlockType::Green => 1,
            BlockType::Yellow => 7,
            BlockType::Border => usize::MAX,
            BlockType::Solution => usize::MAX,
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.block_type.to_string().as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    width: usize,
    height: usize,
    blocks: Vec<Vec<Block>>,
}

impl Map {
    pub fn new(blocks: Vec<Vec<Block>>) -> Self {
        let width = blocks
            .get(0)
            .expect("A map must at least have a height of 1")
            .len();
        let height = blocks.len();

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
        if x < self.width {
            reachable_blocks.push(self.get_block(x + 1, y));
        }
        // To the bottom
        if y < self.height {
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

    pub fn enter_solution(&mut self, locations: &Vec<Block>) {
        self.blocks
            .iter_mut()
            .map(|row| {
                row.iter_mut()
                    .filter(|block| locations.contains(block))
                    .map(|block| block.block_type = BlockType::Solution)
                    .collect_vec()
            })
            .collect_vec();
    }

    pub fn to_string_with_locations(&self, locations: &Vec<Block>, with_numbers: bool) -> String {
        let mut res = "".to_string();
        if with_numbers {
            res += "  ";
            for i in 0..self.width {
                res += &format!("{:>2}", i);
            }
            res += "\n";
        }
        for (i, row) in self.blocks.iter().enumerate() {
            if with_numbers {
                res += &format!("{:>2}", i);
            }
            for block in row {
                let mut block = *block;
                if let Some(_) = locations.iter().find(|location| **location == block) {
                    block.block_type = BlockType::Solution;
                }
                res += &block.to_string();
            }
            res += "\n";
        }
        res
    }

    pub fn to_image(self) -> Option<RgbaImage> {
        let image_width: u32 = self.width as u32 * IMAGE_BLOCK_WIDTH as u32
            + (self.width as u32 - 1) * IMAGE_BORDER_WIDTH as u32;
        let image_height: u32 = self.height as u32 * IMAGE_BLOCK_WIDTH as u32
            + (self.height as u32 - 1) * IMAGE_BORDER_WIDTH as u32;
        let border_rows = (0..IMAGE_BORDER_WIDTH)
            .map(|_| (0..image_width).map(|_| BlockType::Border).collect_vec())
            .collect_vec();
        let buffer_vec = self
            .blocks
            .into_iter()
            .map(|block_row| block_row.iter().map(|block| block.block_type).collect_vec())
            .map(|block_row| expand_block_row(&block_row))
            .intersperse(border_rows)
            .flatten()
            .flatten()
            .map(|block_type| block_type.to_rgba())
            .flatten()
            .collect_vec();

        RgbaImage::from_vec(image_width, image_height, buffer_vec)
    }
}

fn expand_block_row(block_row: &Vec<BlockType>) -> Vec<Vec<BlockType>> {
    let expanded_row = block_row
        .into_iter()
        .intersperse(&BlockType::Border)
        .flat_map(|block_type| {
            if block_type.is_border() {
                0..IMAGE_BORDER_WIDTH
            } else {
                0..IMAGE_BLOCK_WIDTH
            }
            .map(|_| block_type.clone())
        })
        .collect_vec();

    (0..IMAGE_BLOCK_WIDTH)
        .map(|_| expanded_row.clone())
        .collect_vec()
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string_with_locations(&vec![], true))?;
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

impl From<MazeMap> for Map {
    fn from(value: MazeMap) -> Self {
        let mut block_rows = value
            .cells
            .into_iter()
            .flat_map(|cell_row| expand_cell_row(&cell_row))
            .collect_vec();
        block_rows.push(
            (0..(value.width * 2 + 1))
                .map(|i| Block::new(i, value.height, BlockType::Black))
                .collect_vec(),
        );
        Self {
            width: value.width * 2 + 1,
            height: value.height * 2 + 1,
            blocks: block_rows,
        }
    }
}

// Each cell row can be expanded in 3 block rows. One of those is shared between two cell_rows.
// Therefore each cell row gets expanded into two block_row: The top and the middle block row.
fn expand_cell_row(cell_row: &Vec<Cell>) -> Vec<Vec<Block>> {
    let mut block_rows = vec![];
    block_rows.push(get_top_block_row_of_cell_row(cell_row));
    block_rows.push(get_middle_block_row_of_cell_row(cell_row));
    block_rows
}

fn get_top_block_row_of_cell_row(cell_row: &Vec<Cell>) -> Vec<Block> {
    let mut block_row = vec![];

    let y = cell_row
        .first()
        .expect("The MazeMap must at least have a width of 1")
        .y
        * 2
        + 0;

    for cell in cell_row {
        // Top left block is always black
        block_row.push(Block::new(cell.x * 2 + 0, y, BlockType::Black));
        let block_type = if cell.top == Wall::Open {
            BlockType::from(cell.color)
        } else {
            BlockType::Black
        };
        block_row.push(Block::new(cell.x * 2 + 1, y, block_type));
    }

    block_row.push(Block::new(cell_row.len(), y, BlockType::Black));

    block_row
}

fn get_middle_block_row_of_cell_row(cell_row: &Vec<Cell>) -> Vec<Block> {
    let mut block_row = vec![];

    let y = cell_row
        .first()
        .expect("The MazeMap must at least have a width of 1")
        .y
        * 2
        + 1;

    for cell in cell_row {
        let block_type = if cell.left == Wall::Open {
            cell.color.into()
        } else {
            BlockType::Black
        };

        block_row.push(Block::new(cell.x * 2 + 0, y, block_type));

        block_row.push(Block::new(cell.x * 2 + 1, y, cell.color.into()));
    }

    let last_cell = cell_row
        .last()
        .expect("The MazeMap must at least have a width of 1");

    let block_type = if last_cell.right == Wall::Open {
        last_cell.color.into()
    } else {
        BlockType::Black
    };

    block_row.push(Block::new(cell_row.len(), y, block_type));

    block_row
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
        .map(|(block_x, rgba)| Block::new(block_x, block_row_y, BlockType::from_rgba(rgba)))
        .collect_vec()
}
