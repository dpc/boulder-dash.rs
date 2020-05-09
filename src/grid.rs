use rand::{thread_rng, Rng};

use crate::{
    input::{self, Direction},
    map::MapDescription,
};

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct GridPos(usize);

/*
impl GridPos {
    fn to_translation(self) -> Vector3<f32> {
        Vector3::new(self.x as f32 * TILE_SIZE, self.y as f32 * TILE_SIZE, 0.)
    }
}
*/

impl GridPos {
    pub fn new(i: usize) -> Self {
        Self(i)
    }

    pub fn from_xy(x: usize, y: usize, width: usize) -> Self {
        Self(x + y * width)
    }

    pub fn direction(self, d: input::Direction, widht: usize) -> Self {
        use Direction::*;
        match d {
            Up => self.up(widht),
            Down => self.down(widht),
            Left => self.left(),
            Right => self.right(),
        }
    }
    pub fn to_xy(self, width: usize) -> (usize, usize) {
        let x = self.0 % width;
        let y = self.0 / width;

        (x, y)
    }

    pub fn up(self, width: usize) -> Self {
        Self(self.0 + width)
    }

    pub fn down(self, width: usize) -> Self {
        Self(self.0 - width)
    }
    pub fn left(self) -> Self {
        Self(self.0 - 1)
    }
    pub fn right(self) -> Self {
        Self(self.0 + 1)
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
    pub fn to_sprite_number(self) -> Option<usize> {
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
#[derive(Default, Clone)]
pub struct GridState {
    height: usize,
    width: usize,
    pub tiles: Vec<TileType>,
    pub moved: Vec<bool>,

    pub player_pos: GridPos,

    // TODO: move?
    pub diamond_count: usize,
}

impl GridState {
    pub fn new() -> Self {
        let MapDescription {
            height,
            width,
            tiles,
            start,
        } = crate::map::MapDescription::load("./resources/map/01.txt".into())
            .expect("map should load");

        GridState {
            moved: vec![false; tiles.len()],
            height,
            width,
            tiles,
            player_pos: start,
            ..GridState::default()
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn get_tile(&self, pos: GridPos) -> TileType {
        *self.get_tile_ref(pos)
    }

    pub fn set_tile(&mut self, pos: GridPos, v: TileType) {
        *self.get_tile_mut(pos) = v;
    }

    pub fn get_tile_ref(&self, pos: GridPos) -> &TileType {
        &self.tiles[pos.0]
    }
    pub fn get_tile_mut(&mut self, pos: GridPos) -> &mut TileType {
        &mut self.tiles[pos.0]
    }
    pub fn get_moved_mut(&mut self, pos: GridPos) -> &mut bool {
        &mut self.moved[pos.0]
    }

    pub fn get_moved(&mut self, pos: GridPos) -> bool {
        self.moved[pos.0]
    }

    pub fn set_moved(&mut self, pos: GridPos, moved: bool) {
        self.moved[pos.0] = moved;
    }

    fn move_grid_object_if_unmoved(&mut self, src_pos: GridPos, dst_pos: GridPos) {
        if !self.get_moved(src_pos) && !self.get_moved(dst_pos) {
            self.move_grid_object(src_pos, dst_pos);
        }
    }

    // move something from src_pos to dst_pos
    // anything at dst_pos will be destroyed
    fn move_grid_object(&mut self, src_pos: GridPos, dst_pos: GridPos) {
        let src_type = self.get_tile(src_pos);
        self.set_tile(src_pos, TileType::Empty);
        self.set_moved(src_pos, true);
        self.set_tile(dst_pos, src_type);
        self.set_moved(dst_pos, true);

        if self.player_pos == src_pos {
            assert_eq!(src_type, TileType::Player);
            self.player_pos = dst_pos;
        }
    }

    pub fn clean_moved_state(&mut self) {
        for moved in self.moved.iter_mut() {
            *moved = false;
        }
    }

    fn move_player(&mut self, action: Vec<input::Action>) {
        let player_pos = self.player_pos;

        debug_assert!(self.get_tile(self.player_pos).is_player());

        for action in action {
            let dst_pos = player_pos.direction(action.direction, self.width);
            let dst_type = self.get_tile(dst_pos);
            if dst_type.is_empty() {
                self.move_grid_object(player_pos, dst_pos);
                break;
            }
            if dst_type.is_dirt() {
                self.move_grid_object(player_pos, dst_pos);
                break;
            }
            if dst_type.is_diamond() {
                self.move_grid_object(player_pos, dst_pos);
                self.diamond_count += 1;
                break;
            }
            if dst_type.can_be_pushed() {
                let past_dst_pos = dst_pos.direction(action.direction, self.width);
                let past_dst_type = self.get_tile(past_dst_pos);
                if past_dst_type.is_empty() {
                    self.move_grid_object(dst_pos, past_dst_pos);
                    self.move_grid_object(player_pos, dst_pos);
                    break;
                }
            }
        }
    }

    fn move_things_rolling_to_sides(&mut self) {
        for i in 0..self.tiles.len() {
            let pos = GridPos(i);

            if self.get_moved(pos) {
                continue;
            }
            let type_ = self.get_tile(pos);

            if !type_.can_fall() {
                continue;
            }

            let pos_below = pos.down(self.width);

            let type_below = self.get_tile(pos_below);
            if !type_below.can_roll_on_top() {
                continue;
            }

            let pos_left = pos.left();
            let pos_left_down = pos_left.down(self.width);
            let pos_right = pos.right();
            let pos_right_down = pos_right.down(self.width);
            let left_free =
                self.get_tile(pos_left).is_empty() && self.get_tile(pos_left_down).is_empty();
            let right_free =
                self.get_tile(pos_right).is_empty() && self.get_tile(pos_right_down).is_empty();

            if let Some(dst_pos) = match (left_free, right_free) {
                (true, true) => {
                    let mut rng = thread_rng();
                    let choices = [pos_left, pos_right];
                    Some(choices[rng.gen_range(0, choices.len())])
                }
                (true, false) => Some(pos_left),
                (false, true) => Some(pos_right),
                (false, false) => None,
            } {
                self.move_grid_object_if_unmoved(pos, dst_pos);
            }
        }
    }

    fn move_things_falling_down(&mut self) {
        for i in 0..self.tiles.len() {
            let pos = GridPos(i);
            let type_ = self.get_tile(pos);

            if !type_.can_fall() {
                continue;
            }

            let pos_below = pos.down(self.width);
            let type_below = self.get_tile(pos_below);
            if type_below.is_empty() {
                self.move_grid_object_if_unmoved(pos, pos_below);
            }
        }
    }

    pub fn run_tick(&mut self, action: Vec<input::Action>) {
        self.clean_moved_state();
        self.move_player(action);
        self.move_things_rolling_to_sides();
        self.move_things_falling_down();

        /*
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
            } = *self;
            tiles_prev.copy_from(&tiles);
        */
    }
}
