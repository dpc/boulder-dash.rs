use crate::grid;
use amethyst::{
    assets::Handle,
    core::math::Vector3,
    core::timing::Time,
    core::{SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        prelude::*, world::Entities, Entity, Join, Read, System, SystemData, World, Write,
        WriteStorage,
    },
    renderer::{SpriteRender, SpriteSheet},
};
use std::time::Duration;

use crate::input::{self, Direction};
use crate::map::MapDescription;
use crate::TILE_SIZE;

#[derive(Default)]
pub struct GridState {
    pub last_grid_tick: Option<Duration>,

    pub tiles_prev: TileTypeGrid,
    pub tiles: TileTypeGrid,
    pub entities: Grid<Option<Entity>>,
    pub sprites: Option<Handle<SpriteSheet>>,
    pub player_pos: GridPos,

    pub entities_pending_removal: Vec<Entity>,

    pub diamond_count: usize,
}

impl GridState {
    // move something from src_pos to dst_pos
    // anything at dst_pos will be destroyed
    fn move_grid_object_by_src_pos(
        &mut self,
        src_pos: GridPos,
        dst_pos: GridPos,
        storage: &mut WriteStorage<'_, GridObjectState>,
    ) {
        let entity = self.entities.get_mut(src_pos).expect("entity be there");
        let object = storage.get_mut(entity).expect("object be there");
        self.move_grid_object(object, dst_pos);
    }

    fn move_grid_object(&mut self, grid_object: &mut GridObjectState, dst_pos: GridPos) {
        let src_pos = grid_object.pos;

        let entity = self
            .entities
            .get_mut(src_pos)
            .take()
            .expect("entity be there");

        let dst_entity_mut_ref = self.entities.get_mut(dst_pos);
        if let Some(dst_entity) = dst_entity_mut_ref.take() {
            self.entities_pending_removal.push(dst_entity);
        }
        *dst_entity_mut_ref = Some(entity);

        let src_type = self.tiles.get(src_pos);
        *self.tiles.get_mut(src_pos) = TileType::Empty;
        *self.tiles.get_mut(dst_pos) = src_type;
        if self.player_pos == src_pos {
            assert_eq!(src_type, TileType::Player);
            self.player_pos = dst_pos;
        }

        grid_object.moved = true;
        grid_object.pos = dst_pos;
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct GridPos {
    pub x: usize,
    pub y: usize,
}

impl GridPos {
    fn to_translation(self) -> Vector3<f32> {
        Vector3::new(self.x as f32 * TILE_SIZE, self.y as f32 * TILE_SIZE, 0.)
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
        let MapDescription { grid, start } =
            crate::map::MapDescription::load("./resources/map/01.txt".into()).unwrap();
        let mut entities = Grid::new(grid.width, grid.height);

        world.register::<grid::GridObjectState>();
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
                    transform.set_scale(Vector3::new(TILE_SIZE / 32., TILE_SIZE / 32., 1.));
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
            entities,
            tiles_prev: grid.clone(),
            tiles: grid.clone(),
            sprites: Some(sprites.clone()),
            player_pos: start,
            entities_pending_removal: vec![],
            ..GridState::default()
        };
        world.insert(state);
    }

    pub fn deinit(world: &mut World) {
        let mut state = world.remove::<GridState>().expect("state initialized");
        {
        let entities = world.entities();

        for entity in state.entities.vals.drain(..) {
            if let Some(entity) = entity {
                entities.delete(entity).expect("delete should work");
            }
        }
        }
        world.maintain();
    }
}

impl<'s> System<'s> for GridRulesSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, grid::GridObjectState>,
        Read<'s, Time>,
        Write<'s, GridState>,
        Write<'s, input::InputState>,
    );

    fn run(
        &mut self,
        (entitites, mut transforms, mut grid_objects, time, mut grid_map_state, mut input_state): Self::SystemData,
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
        for object in (&mut grid_objects).join() {
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
                grid_map_state.move_grid_object(object, pos_below);
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

            if let Some(dst_pos) = match (left_free, right_free) {
                (true, true) => Some(pos_left), // TODO: randomize?
                (true, false) => Some(pos_left),
                (false, true) => Some(pos_right),
                (false, false) => None,
            } {
                grid_map_state.move_grid_object(object, dst_pos);
            }
        }

        let player_pos = grid_map_state.player_pos;

        debug_assert!(grid_map_state.tiles.get(player_pos).is_player());

        for action in input_state.pop_action() {
            let dst_pos = player_pos.direction(action.direction);
            let dst_type = grid_map_state.tiles.get(dst_pos);
            if dst_type.is_empty() {
                grid_map_state.move_grid_object_by_src_pos(player_pos, dst_pos, &mut grid_objects);
                break;
            }
            if dst_type.is_dirt() {
                grid_map_state.move_grid_object_by_src_pos(player_pos, dst_pos, &mut grid_objects);
                break;
            }
            if dst_type.is_diamond() {
                grid_map_state.move_grid_object_by_src_pos(player_pos, dst_pos, &mut grid_objects);
                grid_map_state.diamond_count += 1;
                break;
            }
            if dst_type.can_be_pushed() {
                let past_dst_pos = dst_pos.direction(action.direction);
                let past_dst_type = grid_map_state.tiles.get(past_dst_pos);
                if past_dst_type.is_empty() {
                    grid_map_state.move_grid_object_by_src_pos(
                        dst_pos,
                        past_dst_pos,
                        &mut grid_objects,
                    );
                    grid_map_state.move_grid_object_by_src_pos(
                        player_pos,
                        dst_pos,
                        &mut grid_objects,
                    );
                    break;
                }
            }
        }

        for entity in grid_map_state.entities_pending_removal.drain(..) {
            entitites.delete(entity).expect("not to fail");
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

    fn is_dirt(self) -> bool {
        use TileType::*;
        match self {
            Dirt => true,
            _ => false,
        }
    }
    fn is_diamond(self) -> bool {
        use TileType::*;
        match self {
            Diamond => true,
            _ => false,
        }
    }
    fn can_be_pushed(self) -> bool {
        use TileType::*;
        match self {
            Rock => true,
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
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            vals: vec![Default::default(); width * height],
        }
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn get(&self, pos: GridPos) -> T {
        self.get_ref(pos).clone()
    }

    pub fn set(&mut self, pos: GridPos, v: T) {
        *self.get_mut(pos) = v;
    }

    pub fn get_ref(&self, pos: GridPos) -> &T {
        &self.vals[pos.x + pos.y * self.width]
    }
    pub fn get_mut(&mut self, pos: GridPos) -> &mut T {
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
