use crate::maze::{Cell, Maze, MazeError};

use std::{io, str::FromStr};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub struct ClientConnection {
    rx: BufReader<OwnedReadHalf>,
    tx: OwnedWriteHalf,
    maze: Maze,
    current_position: (isize, isize),
    view_radius: usize,
}

#[derive(Error, Debug)]
pub enum ClientConnectionError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[allow(unused)]
    #[error("maze error: {0}")]
    Maze(MazeError),

    #[error("unknown command: {0}")]
    CommandParseError(String),
}

pub type Result<T> = std::result::Result<T, ClientConnectionError>;

#[derive(Debug, Clone)]
enum Command {
    MoveRight,
    MoveLeft,
    MoveUp,
    MoveDown,
    Refresh,
}

impl FromStr for Command {
    type Err = ClientConnectionError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "left" | "a" => Ok(Command::MoveLeft),
            "right" | "d" => Ok(Command::MoveRight),
            "up" | "w" => Ok(Command::MoveUp),
            "down" | "s" => Ok(Command::MoveDown),
            "" => Ok(Command::Refresh),
            other => Err(ClientConnectionError::CommandParseError(other.to_owned())),
        }
    }
}

impl ClientConnection {
    pub fn new(stream: TcpStream) -> Self {
        let (rx, tx) = stream.into_split();
        let rx = BufReader::new(rx);

        Self {
            rx,
            tx,
            maze: include_str!("maze.txt").parse().unwrap(),
            current_position: (1, 1),
            view_radius: 6,
        }
    }

    async fn write_view_in_maze(&mut self) -> Result<()> {
        // Our view is a circle around the current position
        let radius = self.view_radius as isize;
        let (px, py) = self.current_position;
        let maze_view = (-radius..=radius)
            .map(|row| {
                let maze_row = (-radius..=radius)
                    .map(|col| {
                        let dist = ((row.pow(2) + col.pow(2)) as f32).sqrt();
                        let (x, y) = (px + col, py + row);
                        if row == 0 && col == 0 {
                            '@'
                        } else if dist < (radius as f32) && self.maze.in_bounds(x, y) {
                            match self
                                .maze
                                .get_cell(x as usize, y as usize)
                                .unwrap_or(&Cell::Empty)
                            {
                                Cell::Filled => '#',
                                Cell::Goal => '*',
                                Cell::Empty => ' ',
                            }
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>();
                format!("{}\n", maze_row)
            })
            .collect::<String>();

        // Write view to client
        self.tx.write_all(&maze_view.into_bytes()).await?;
        self.tx.write_all("\n".as_bytes()).await?;
        Ok(())
    }

    async fn print_interface(&mut self) -> Result<()> {
        self.tx
            .write_all("~~ MAZE MASTER ~~\n\n".as_bytes())
            .await?;
        self.write_view_in_maze().await?;
        self.tx
            .write_all("\navailable commands: up, down, left, right (or use WASD)\n\n".as_bytes())
            .await?;
        Ok(())
    }

    async fn read_command(&mut self) -> Result<Command> {
        let mut line = Default::default();
        self.rx.read_line(&mut line).await?;
        line = line.strip_suffix('\n').unwrap_or(&line).to_owned();
        line.parse()
    }

    async fn try_move(&mut self, dx: isize, dy: isize) -> Result<()> {
        // Attempt to move
        let (x, y) = self.current_position;
        let (tx, ty) = (x + dx, y + dy);
        if self.maze.in_bounds(tx, ty) {
            if let Some(target_cell) = self.maze.get_cell(tx as usize, ty as usize) {
                if target_cell.is_traversable() {
                    self.current_position = (tx, ty);
                }
            }
        }

        // Re-render ui
        self.print_interface().await?;

        Ok(())
    }

    async fn apply_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Refresh => self.print_interface().await,
            Command::MoveLeft => self.try_move(-1, 0).await,
            Command::MoveRight => self.try_move(1, 0).await,
            Command::MoveUp => self.try_move(0, -1).await,
            Command::MoveDown => self.try_move(0, 1).await,
        }
    }

    pub async fn handle(mut self) -> Result<()> {
        // Print interface
        self.print_interface().await?;

        loop {
            self.tx.write_all("> ".as_bytes()).await?;
            // Read command
            match self.read_command().await {
                Ok(command) => {
                    self.tx.write_all("\n".as_bytes()).await?;
                    self.apply_command(command).await?;
                }
                Err(err) => {
                    self.tx.write_all(format!("{}\n", err).as_bytes()).await?;
                }
            };
        }
    }
}
