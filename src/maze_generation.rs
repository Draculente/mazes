use rand::{
    distributions::{Distribution, Standard},
    seq::SliceRandom,
    Rng,
};

use anyhow::{anyhow, Ok};

/// The entered loop_prop is divided by this factor, to create a behavior in which 1 is almost total connection and 0 is no loops.
const LOOP_PROB_FACTOR: f64 = 6.20;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum Wall {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Blue,
    Orange,
    Yellow,
    Green,
}

impl Distribution<Color> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Color {
        match rng.gen_range(0..4) {
            0 => Color::Blue,
            1 => Color::Orange,
            2 => Color::Yellow,
            _ => Color::Green,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub top: Wall,
    pub right: Wall,
    pub bottom: Wall,
    pub left: Wall,
    pub x: usize,
    pub y: usize,
    pub color: Color,
}

impl Cell {
    fn new(x: usize, y: usize) -> Self {
        Self {
            top: Wall::Closed,
            right: Wall::Closed,
            bottom: Wall::Closed,
            left: Wall::Closed,
            x,
            y,
            color: rand::random(),
        }
    }

    fn set_color(&mut self, color: Color) {
        self.color = color
    }

    /// The relation of the other cell to self (e.g. Relation::Top means, that other is on top of self)
    fn relation(&self, other: &Cell) -> anyhow::Result<Relation> {
        if other.x == self.x + 1 && other.y == self.y {
            return Ok(Relation::Right);
        }
        if other.y == self.y + 1 && other.x == self.x {
            return Ok(Relation::Bottom);
        }
        if self.x > 0 && other.x == self.x - 1 && other.y == self.y {
            return Ok(Relation::Left);
        }
        if self.y > 0 && other.y == self.y - 1 && other.x == self.x {
            return Ok(Relation::Top);
        }
        Err(anyhow!("The other cell is not a neighbor to self"))
    }

    fn open_wall_to(&mut self, other: &Cell) -> anyhow::Result<()> {
        match self.relation(other)? {
            Relation::Top => self.top = Wall::Open,
            Relation::Right => self.right = Wall::Open,
            Relation::Bottom => self.bottom = Wall::Open,
            Relation::Left => self.left = Wall::Open,
        };
        Ok(())
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Relation {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug)]
pub struct MazeMap {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
}

impl MazeMap {
    fn new(width: usize, height: usize) -> Self {
        let mut cells = vec![];

        for y in 0..height {
            let mut row = vec![];
            for x in 0..width {
                row.push(Cell::new(x, y));
            }
            cells.push(row);
        }

        Self {
            cells,
            width,
            height,
        }
    }

    fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        self.cells.get(y).and_then(|row| row.get(x))
    }

    fn get_cell_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        self.cells.get_mut(y).and_then(|row| row.get_mut(x))
    }

    fn get_neighbors(&self, cell: &Cell) -> Vec<Cell> {
        let mut neighbors = vec![];

        // To the left
        if cell.x > 0 {
            neighbors.push(self.get_cell(cell.x - 1, cell.y));
        }
        // To the top
        if cell.y > 0 {
            neighbors.push(self.get_cell(cell.x, cell.y - 1));
        }
        // To the right
        if cell.x < self.width {
            neighbors.push(self.get_cell(cell.x + 1, cell.y));
        }
        // To the bottom
        if cell.y < self.height {
            neighbors.push(self.get_cell(cell.x, cell.y + 1));
        }

        neighbors
            .into_iter()
            .filter(|cell| cell.is_some())
            .map(|c_opt| c_opt.map(|cell| cell.clone()))
            .collect::<Option<Vec<_>>>()
            .expect("Each cell should have at least 2 neighbors")
    }

    fn connect_cells(&mut self, cell_a: &Cell, cell_b: &Cell) -> anyhow::Result<()> {
        self.get_cell_mut(cell_a.x, cell_a.y)
            .ok_or(anyhow!("Cell_A is not a part of the map"))?
            .open_wall_to(cell_b)?;

        self.get_cell_mut(cell_b.x, cell_b.y)
            .ok_or(anyhow!("Cell_B is not a part of the map"))?
            .open_wall_to(cell_a)?;

        Ok(())
    }
}

/// https://en.wikipedia.org/wiki/Maze_generation_algorithm#Iterative_implementation_(with_stack)
pub fn generate_maze(
    width: usize,
    height: usize,
    loop_prob: Option<f64>,
) -> anyhow::Result<MazeMap> {
    let mut map = MazeMap::new(width, height);
    let first_cell = map
        .get_cell(0, 0)
        .ok_or(anyhow!("The maze must at least have the dimensions 1x1"))?;
    let mut stack = vec![first_cell.clone()];
    let mut visited = vec![first_cell.clone()];
    let mut color: Color = rand::random();
    let mut rng = rand::thread_rng();

    while let Some(current_cell) = stack.pop() {
        let unvisited_neighbors: Vec<Cell> = map
            .get_neighbors(&current_cell)
            .into_iter()
            .filter(|cell| {
                !visited.contains(cell)
                    || rng.gen_bool(
                        loop_prob
                            .filter(|f| *f != 0.0)
                            .map(|f| f / LOOP_PROB_FACTOR)
                            .unwrap_or(0.0),
                    )
            })
            .collect();

        if !unvisited_neighbors.is_empty() {
            stack.push(current_cell);
            let chosen_cell = unvisited_neighbors
                .choose(&mut rand::thread_rng())
                .expect("The get_neighbors can't be empty");
            map.connect_cells(&current_cell, &chosen_cell)?;
            map.get_cell_mut(current_cell.x, current_cell.y)
                .map(|cell| cell.set_color(color));
            visited.push(chosen_cell.clone());
            stack.push(chosen_cell.clone());
        } else {
            color = rand::random();
        }
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_right_corner_has_two_neighbors() {
        let map = MazeMap::new(3, 3);
        assert_eq!(map.get_neighbors(&Cell::new(2, 0)).len(), 2);
    }

    #[test]
    fn bottom_left_corner_has_two_neighbors() {
        let map = MazeMap::new(3, 3);
        assert_eq!(map.get_neighbors(&Cell::new(0, 2)).len(), 2);
    }

    #[test]
    fn middle_left_cell_has_three_neighbors() {
        let map = MazeMap::new(3, 3);
        assert_eq!(map.get_neighbors(&Cell::new(0, 1)).len(), 3);
    }

    #[test]
    fn middle_cell_has_four_neighbors() {
        let map = MazeMap::new(3, 3);
        assert_eq!(map.get_neighbors(&Cell::new(1, 1)).len(), 4);
    }

    #[test]
    fn relation_top_neighbor() {
        let cell_a = Cell::new(1, 1);
        let cell_b = Cell::new(1, 2);
        assert_eq!(cell_a.relation(&cell_b).unwrap(), Relation::Bottom);
    }

    #[test]
    fn relation_right_neighbor() {
        let cell_a = Cell::new(1, 1);
        let cell_b = Cell::new(2, 1);
        assert_eq!(cell_a.relation(&cell_b).unwrap(), Relation::Right);
    }

    #[test]
    fn relation_bottom_neighbor() {
        let cell_a = Cell::new(1, 1);
        let cell_b = Cell::new(1, 0);
        assert_eq!(cell_a.relation(&cell_b).unwrap(), Relation::Top);
    }

    #[test]
    fn relation_left_neighbor() {
        let cell_a = Cell::new(1, 1);
        let cell_b = Cell::new(0, 1);
        assert_eq!(cell_a.relation(&cell_b).unwrap(), Relation::Left);
    }

    #[test]
    fn relation_non_neighbor() {
        let cell_a = Cell::new(1, 1);
        let cell_b = Cell::new(3, 2);
        assert!(cell_a.relation(&cell_b).is_err());
    }

    #[test]
    fn connect_cells_top_neighbor() {
        let mut map = MazeMap::new(3, 3);
        let cell_a = Cell::new(1, 1);
        let cell_b = Cell::new(1, 2);
        map.connect_cells(&cell_a, &cell_b).unwrap();
        assert_eq!(map.get_cell(1, 1).unwrap().top, Wall::Closed);
        assert_eq!(map.get_cell(1, 1).unwrap().bottom, Wall::Open);
        assert_eq!(map.get_cell(1, 2).unwrap().bottom, Wall::Closed);
        assert_eq!(map.get_cell(1, 2).unwrap().top, Wall::Open);
    }

    #[test]
    fn connect_cells_right_neighbor() {
        let mut map = MazeMap::new(3, 3);
        let cell_a = Cell::new(1, 1);
        let cell_b = Cell::new(2, 1);
        map.connect_cells(&cell_a, &cell_b).unwrap();
        assert_eq!(map.get_cell(1, 1).unwrap().right, Wall::Open);
        assert_eq!(map.get_cell(2, 1).unwrap().left, Wall::Open);
    }

    #[test]
    fn connect_cells_bottom_neighbor() {
        let mut map = MazeMap::new(3, 3);
        let cell_a = Cell::new(1, 1);
        let cell_b = Cell::new(1, 0);
        map.connect_cells(&cell_a, &cell_b).unwrap();
        assert_eq!(map.get_cell(1, 1).unwrap().bottom, Wall::Closed);
        assert_eq!(map.get_cell(1, 1).unwrap().top, Wall::Open);
        assert_eq!(map.get_cell(1, 0).unwrap().bottom, Wall::Open);
        assert_eq!(map.get_cell(1, 0).unwrap().top, Wall::Closed);
    }

    #[test]
    fn connect_cells_left_neighbor() {
        let mut map = MazeMap::new(3, 3);
        let cell_a = Cell::new(1, 1);
        let cell_b = Cell::new(0, 1);
        map.connect_cells(&cell_a, &cell_b).unwrap();
        assert_eq!(map.get_cell(1, 1).unwrap().left, Wall::Open);
        assert_eq!(map.get_cell(0, 1).unwrap().right, Wall::Open);
    }
}
