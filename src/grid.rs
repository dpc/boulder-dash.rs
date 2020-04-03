use crate::grid;
use amethyst::{
    assets::Handle,
    core::math::Vector3,
    core::timing::Time,
    core::{SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        prelude::{Component, DenseVecStorage},
        Entity, Join, Read, System, SystemData, World, Write, WriteStorage,
    },
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};
use anyhow::{bail, format_err, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;

use crate::input::{self, Direction};

#[derive(Default)]
pub struct GridState {
    pub last_grid_tick: Option<Duration>,

    pub tiles_prev: TileTypeGrid,
    pub tiles: TileTypeGrid,
    pub entities: Grid<Option<Entity>>,
    pub sprites: Option<Handle<SpriteSheet>>,
    pub player_pos: GridPos,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct GridPos {
    pub x: usize,
    pub y: usize,
}

impl GridPos {
    fn to_translation(self) -> Vector3<f32> {
        Vector3::new(self.x as f32 * 32., self.y as f32 * 32., 0.)
    }
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

    pub fn direction(self, d: input::Direction) -> Self {
        use Direction::*;
        match d {
            Up => self.up(),
            Down => self.down(),
            Left => self.left(),
            Right => self.right(),
        }
    }

    pub fn up(self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
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
pub struct GridRulesSystem;

impl GridRulesSystem {
    pub fn init(world: &mut World, sprites: Handle<SpriteSheet>) {
        let LoadMapData { grid, start } = load_map("./resources/map/01.txt".into()).unwrap();
        let mut entities = Grid::new(grid.width, grid.height);

        for y in 0..grid.height {
            for x in 0..grid.width {
                let pos = GridPos { x, y };
                let t = grid.get(pos);
                if let Some(sprite_number) = t.to_sprite_number() {
                    let sprite_render = SpriteRender {
                        sprite_sheet: sprites.clone(),
                        sprite_number,
                    };

                    let mut transform = Transform::default();
                    transform.set_translation(pos.to_translation());
                    let entity = world
                        .create_entity()
                        .with(sprite_render)
                        .with(GridObjectState::new(x, y))
                        .with(transform)
                        .build();

                    entities.set(pos, Some(entity));
                }
            }
        }

        let state = GridState {
            last_grid_tick: Default::default(),
            entities,
            tiles_prev: grid.clone(),
            tiles: grid.clone(),
            sprites: Some(sprites.clone()),
            player_pos: start,
        };
        world.register::<grid::GridObjectState>();
        world.insert(state);
    }
}

impl<'s> System<'s> for GridRulesSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, grid::GridObjectState>,
        Read<'s, Time>,
        Write<'s, GridState>,
        Write<'s, input::InputState>,
    );

    fn run(
        &mut self,
        (mut transforms, mut grid_objects, time, mut grid_map_state, mut input_state): Self::SystemData,
    ) {
        let do_grid_tick = grid_map_state
            .last_grid_tick
            .map(|last| Duration::from_millis(125) < time.absolute_time() - last)
            .unwrap_or(true);

        if do_grid_tick {
            grid_map_state.last_grid_tick = Some(time.absolute_time());
        } else {
            return;
        }

        // objects falling straight down
        for mut object in (&mut grid_objects).join() {
            let GridState {
                ref tiles_prev,
                ref mut tiles,
                ref mut entities,
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
                entities.swap(object.pos, pos_below);
                // *tiles.get_mut(*pos) = type_below;
                // *tiles.get_mut(pos_below) = type_;
                object.pos = pos_below;
                object.moved = true;
            }
        }

        // objects rolling to sides
        for object in (&mut grid_objects).join() {
            if object.moved {
                continue;
            }

            let GridState {
                ref tiles_prev,
                ref mut tiles,
                ref mut entities,
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
                entities.swap(object.pos, move_pos);
                object.pos = move_pos;
                object.moved = true;
            }
        }

        let player_pos = grid_map_state.player_pos;
        let player_entity = grid_map_state
            .entities
            .get(player_pos)
            .expect("player entity there");

        let player = grid_objects
            .get_mut(player_entity)
            .expect("player by entity there");

        debug_assert!(grid_map_state.tiles.get(player_pos).is_player());

        if let Some(action) = input_state.pop_action() {
            let dst_pos = player_pos.direction(action.direction);
            let dst_type = grid_map_state.tiles.get(dst_pos);
            if dst_type.is_empty() {
                grid_map_state.tiles.swap(player_pos, dst_pos);
                grid_map_state.entities.swap(player_pos, dst_pos);
                player.pos = dst_pos;
                player.moved = true;
                grid_map_state.player_pos = dst_pos;
            }
        }

        for (object, transform) in (&mut grid_objects, &mut transforms).join() {
            if !object.moved {
                continue;
            }

            object.moved = false;
            transform.set_translation(object.pos.to_translation());
        }
        let GridState {
            ref mut tiles_prev,
            ref mut tiles,
            ..
        } = *grid_map_state;
        tiles_prev.copy_from(&tiles);
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

impl Default for TileType {
    fn default() -> Self {
        TileType::Empty
    }
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
    fn is_player(self) -> bool {
        use TileType::*;
        match self {
            Player => true,
            _ => false,
        }
    }
}

pub type TileTypeGrid = Grid<TileType>;

#[derive(Default, Clone)]
pub struct Grid<T> {
    height: usize,
    width: usize,
    vals: Vec<T>,
}

impl<T> Grid<T>
where
    T: Clone + Default,
{
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            vals: vec![Default::default(); width * height],
        }
    }
    fn swap(&mut self, p1: GridPos, p2: GridPos) {
        let tmp = self.get(p1);
        *self.get_mut(p1) = self.get(p2);
        *self.get_mut(p2) = tmp;
    }

    fn get(&self, pos: GridPos) -> T {
        self.get_ref(pos).clone()
    }

    fn set(&mut self, pos: GridPos, v: T) {
        *self.get_mut(pos) = v;
    }

    fn get_ref(&self, pos: GridPos) -> &T {
        &self.vals[pos.x + pos.y * self.width]
    }
    fn get_mut(&mut self, pos: GridPos) -> &mut T {
        &mut self.vals[pos.x + pos.y * self.width]
    }
}

impl<T> Grid<T>
where
    T: Copy,
{
    fn copy_from(&mut self, other: &Self) {
        debug_assert_eq!(self.width, other.width);
        debug_assert_eq!(self.height, other.height);

        self.vals[..].copy_from_slice(&other.vals);
    }
}

struct LoadMapData {
    grid: TileTypeGrid,
    start: GridPos,
}

fn load_map(path: PathBuf) -> Result<LoadMapData> {
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

    Ok(LoadMapData {
        grid,
        start: start.ok_or_else(|| format_err!("No start position found"))?,
    })
}
