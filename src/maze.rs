use std::str::FromStr;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MazeError {
    #[error("failed to parse maze format: {0}")]
    MazeParsingFailed(String),
}

pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy)]
pub enum Cell {
    Filled,
    Empty,
    Goal,
}

impl Cell {
    pub fn is_traversable(&self) -> bool {
        matches!(self, Cell::Empty | Cell::Goal)
    }
}

impl TryFrom<char> for Cell {
    type Error = MazeError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '#' => Ok(Cell::Filled),
            '.' => Ok(Cell::Empty),
            '*' => Ok(Cell::Goal),
            c => Err(MazeError::MazeParsingFailed(format!(
                "unknown character: {}",
                c
            ))),
        }
    }
}

impl Maze {
    pub fn in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < (self.width as isize) && y >= 0 && y < (self.height as isize)
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        self.cells.get(y * self.width + x)
    }
}

impl FromStr for Maze {
    type Err = MazeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Peek the first line to determine width
        let mut lines = s.lines().peekable();
        let width = lines
            .peek()
            .ok_or(MazeError::MazeParsingFailed("no first line".to_owned()))?
            .len();
        let cells: Vec<Cell> = lines
            .flat_map(|row| row.chars().map(|c| c.try_into().unwrap())) // TODO: dont unwrap
            .collect();

        Ok(Self {
            width,
            height: cells.len() / width,
            cells,
        })
    }
}

impl Default for Maze {
    fn default() -> Self {
        let width = 21;
        let height = 21;
        let cells = vec![Cell::Filled; width * height];
        Self {
            width,
            height,
            cells,
        }
    }
}
