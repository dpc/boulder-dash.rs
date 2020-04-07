use anyhow::{bail, format_err, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use crate::grid::{GridPos, TileType, TileTypeGrid};

pub struct MapDescription {
    pub grid: TileTypeGrid,
    pub start: GridPos,
}

impl MapDescription {
    pub fn load(path: PathBuf) -> Result<Self> {
        let mut start = None;

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let lines: Result<Vec<String>> = reader.lines().map(|e| e.map_err(|e| e.into())).collect();
        let lines = lines?;
        let width = lines[0].len();
        let height = lines.len();

        let mut grid = TileTypeGrid::new(width, height);

        for (y, line) in lines.iter().rev().enumerate() {
            if width != line.len() {
                bail!("Lines not equal len");
            }

            for (x, ch) in line.chars().enumerate() {
                let pos = GridPos::new(x, y);
                grid.set(
                    pos,
                    match ch {
                        's' => {
                            if start.is_some() {
                                bail!("Multiple starting positions found");
                            }
                            start = Some(pos);
                            TileType::Player
                        }
                        '#' => TileType::Steel,
                        '%' => TileType::Wall,
                        '.' => TileType::Dirt,
                        'o' => TileType::Rock,
                        '*' => TileType::Diamond,
                        _ => TileType::Empty,
                    },
                );
            }
        }

        Ok(MapDescription {
            grid,
            start: start.ok_or_else(|| format_err!("No start position found"))?,
        })
    }
}
