use crate::grid;
use amethyst::{
    assets::Handle,
    core::timing::Time,
    core::{SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        prelude::{Component, DenseVecStorage},
        Join, Read, System, SystemData, World, Write, WriteStorage,
    },
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};
use anyhow::{bail, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Default)]
pub struct GridState {
    pub last_tick: Option<Duration>,
    pub tiles_prev: TileTypeGrid,
    pub tiles: TileTypeGrid,
    pub sprites: Option<Handle<SpriteSheet>>,
    pub player_pos: GridPos,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct GridPos {
    pub x: usize,
    pub y: usize,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct GridObjectState {
    pub pos: GridPos,
    pub moved: bool,
}
impl GridObjectState {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            pos: GridPos::new(x, y),
            moved: false,
        }
    }
}
impl GridPos {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn down(self) -> Self {
        Self {
            x: self.x,
            y: self.y - 1,
        }
    }
    pub fn left(self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y,
        }
    }
    pub fn right(self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y,
        }
    }
}

impl Component for GridObjectState {
    type Storage = DenseVecStorage<Self>;
}

#[derive(SystemDesc)]
pub struct GridObjectSystem;

impl<'s> System<'s> for GridObjectSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, GridObjectState>,
        Read<'s, Time>,
        Write<'s, GridState>,
    );

    fn run(
        &mut self,
        (mut transforms, mut grid_objects, time, mut grid_map_state): Self::SystemData,
    ) {
        if grid_map_state
            .last_tick
            .map(|last| time.absolute_time() - last < Duration::from_millis(250))
            .unwrap_or(false)
        {
            return;
        } else {
            grid_map_state.last_tick = Some(time.absolute_time());
        }

        for mut object in (&mut grid_objects).join() {
            object.moved = false;

            // objects falling straight down
            let GridState {
                ref tiles_prev,
                ref mut tiles,
                ..
            } = *grid_map_state;

            let type_ = tiles.get(object.pos);

            if !type_.can_fall() {
                continue;
            }

            let pos_below = object.pos.down();
            let type_below = tiles.get(pos_below);
            let type_below_prev = tiles_prev.get(pos_below);
            if type_below.is_empty() && type_below_prev.is_empty() {
                tiles.swap(object.pos, pos_below);
                // *tiles.get_mut(*pos) = type_below;
                // *tiles.get_mut(pos_below) = type_;
                object.pos = pos_below;
                object.moved = true;
            }
        }

        for object in (&mut grid_objects).join() {
            if object.moved {
                continue;
            }

            let GridState {
                ref tiles_prev,
                ref mut tiles,
                ..
            } = *grid_map_state;

            let type_ = tiles.get(object.pos);

            if !type_.can_fall() {
                continue;
            }

            let pos_below = object.pos.down();

            let type_below = tiles.get(pos_below);
            if !type_below.can_roll_on_top() {
                continue;
            }

            let pos_left = object.pos.left();
            let pos_left_down = pos_left.down();
            let pos_right = object.pos.right();
            let pos_right_down = pos_right.down();
            let left_free = tiles.get(pos_left).is_empty()
                && tiles_prev.get(pos_left).is_empty()
                && tiles.get(pos_left_down).is_empty()
                && tiles_prev.get(pos_left_down).is_empty();
            let right_free = tiles.get(pos_right).is_empty()
                && tiles_prev.get(pos_right).is_empty()
                && tiles.get(pos_right_down).is_empty()
                && tiles_prev.get(pos_right_down).is_empty();

            if let Some(move_pos) = match (left_free, right_free) {
                (true, true) => Some(pos_left), // TODO: randomize?
                (true, false) => Some(pos_left),
                (false, true) => Some(pos_right),
                (false, false) => None,
            } {
                tiles.swap(object.pos, move_pos);
                object.pos = move_pos;
                object.moved = true;
            }
        }

        for (object, transform) in (&mut grid_objects, &mut transforms).join() {
            transform.set_translation_y(object.pos.y as f32 * 32.);
            transform.set_translation_x(object.pos.x as f32 * 32.);
        }
        let GridState {
            ref mut tiles_prev,
            ref mut tiles,
            ..
        } = *grid_map_state;
        // std::mem::swap(tiles_prev, tiles);
        tiles_prev.tiles[..].copy_from_slice(&tiles.tiles);
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TileType {
    Empty,
    Player,
    Dirt,
    Rock,
    Wall,
    Diamond,
    Steel,
}

impl TileType {
    fn to_sprite_number(self) -> Option<usize> {
        use TileType::*;
        Some(match self {
            Empty => return None,
            Player => 0,
            Dirt => 2,
            Rock => 3,
            Steel => 4,
            Wall => 5,
            Diamond => 6,
        })
    }

    fn can_fall(self) -> bool {
        use TileType::*;
        match self {
            Rock => true,
            Diamond => true,
            _ => false,
        }
    }

    fn can_roll_on_top(self) -> bool {
        use TileType::*;
        match self {
            Rock => true,
            Diamond => true,
            Wall => true,
            _ => false,
        }
    }

    fn is_empty(self) -> bool {
        use TileType::*;
        match self {
            Empty => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone)]
pub struct TileTypeGrid {
    height: usize,
    width: usize,
    tiles: Vec<TileType>,
}

impl TileTypeGrid {
    fn get(&self, pos: GridPos) -> TileType {
        *self.get_ref(pos)
    }

    fn swap(&mut self, p1: GridPos, p2: GridPos) {
        let tmp = self.get(p1);
        *self.get_mut(p1) = self.get(p2);
        *self.get_mut(p2) = tmp;
    }

    fn get_ref(&self, pos: GridPos) -> &TileType {
        &self.tiles[pos.x + (self.height - pos.y - 1) * self.width]
    }
    fn get_mut(&mut self, pos: GridPos) -> &mut TileType {
        &mut self.tiles[pos.x + (self.height - pos.y - 1) * self.width]
    }
}

struct LoadMapData {
    grid: TileTypeGrid,
    start: GridPos,
}

fn load_map(path: PathBuf) -> Result<LoadMapData> {
    let mut width = None;
    let mut height = 0;
    let mut start = None;
    let mut tiles = vec![];

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (y, line) in reader.lines().enumerate() {
        let line = line?;
        if let Some(width) = width {
            if width != line.len() {
                bail!("Lines not equal len");
            }
        } else {
            width = Some(line.len());
        }
        height = y + 1;

        for (x, ch) in line.chars().enumerate() {
            tiles.push(match ch {
                's' => {
                    start = Some(GridPos::new(x, y));
                    TileType::Player
                }
                '#' => TileType::Steel,
                '%' => TileType::Wall,
                '.' => TileType::Dirt,
                'o' => TileType::Rock,
                '*' => TileType::Diamond,
                _ => TileType::Empty,
            });
        }
    }

    Ok(LoadMapData {
        grid: TileTypeGrid {
            width: width.unwrap(),
            height,
            tiles,
        },
        start: start.unwrap(),
    })
}

pub fn init(world: &mut World, sprites: Handle<SpriteSheet>) {
    let LoadMapData { grid, start } = load_map("./resources/map/01.txt".into()).unwrap();

    let state = GridState {
        last_tick: Default::default(),
        tiles_prev: grid.clone(),
        tiles: grid.clone(),
        sprites: Some(sprites.clone()),
        player_pos: start,
    };

    for y in 0..grid.height {
        for x in 0..grid.width {
            let t = grid.get(GridPos { x, y });
            if let Some(sprite_number) = t.to_sprite_number() {
                let sprite_render = SpriteRender {
                    sprite_sheet: sprites.clone(),
                    sprite_number,
                };

                world
                    .create_entity()
                    .with(sprite_render)
                    .with(GridObjectState::new(x, y))
                    .with(Transform::default())
                    .build();
            }
        }
    }

    world.register::<grid::GridObjectState>();
    world.insert(state);
}
