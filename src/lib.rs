mod utils;

extern crate web_sys;

use wasm_bindgen::prelude::*;
use std::str::FromStr;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Inactive = 0,
    Active = 1,
}

#[wasm_bindgen]
pub struct Universe {
    map: Map,
    cells: Vec<Cell>,

    // TODO: Player should be struct
    player_pos: Position,
    player_direction: Direction,

    bullets: Vec<Bullet>,
    cur_bullet_index: usize,

    asteriods: Vec<Asteriod>,
}

#[derive(Clone)]
struct Bullet {
    active: bool,
    position: Position,
    direction: Direction,
}

impl Bullet {
    fn new() -> Bullet {
        let direction = Direction::from_str("up").unwrap();
        Bullet{
            active: true,
            position: Position{x: 0, y: 0},
            direction: direction,
        }
    }
}

struct Asteriod {
    position: Position,
    radius: u32
}

pub struct Map {
    height: u32,
    width: u32,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Map {
        Map{
            width: width,
            height: height
        }
    }

    fn get_index(&self, column: u32, row: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn get_bullet_index(&self, bullet: &Bullet) -> usize {
        self.get_index(bullet.position.x, bullet.position.y)
    }

    fn get_asteroid_indexes(&self, asteriod: &Asteriod) -> Vec<usize>{
        let mut indexes = Vec::new();
        for y in (asteriod.position.y - asteriod.radius)..(asteriod.position.y + asteriod.radius + 1) {
            for x in (asteriod.position.x - asteriod.radius)..(asteriod.position.x + asteriod.radius + 1) {
                indexes.push(self.get_index_from_position(&Position{x: x, y: y}))
            }
        }
        indexes
    }
    
    fn position_in_direction(&self, position: &Position, direction: &Direction) -> Position {
        let new_position: Position;
        match direction {
            Direction::Up => {
                if position.y == 0 {
                    new_position = Position{
                        x: position.x, 
                        y: position.y
                    }
                } else {
                    new_position = Position{
                        x: position.x, 
                        y: position.y - 1
                    }
                }
            },
            Direction::Right => {
                if position.x == self.width - 1 {
                    new_position = Position{
                        x: position.x, 
                        y: position.y
                    }
                } else {
                    new_position = Position{
                        x: position.x + 1, 
                        y: position.y
                    }
                }
            },
            Direction::Down => {
                if position.y == self.height - 1 {
                    new_position = Position{
                        x: position.x, 
                        y: position.y
                    }
                } else {
                    new_position = Position{
                        x: position.x, 
                        y: position.y + 1
                    }
                }
            },
            Direction::Left => {
                if position.x == 0 {
                    new_position = Position{
                        x: position.x, 
                        y: position.y
                    }
                } else {
                    new_position = Position{
                        x: position.x - 1, 
                        y: position.y
                    }
                }
            }
        }
        new_position
    }
    
    // Needs to check if index is outside map
    fn get_index_from_position(&self, position: &Position) -> usize {
        if position.x > self.width - 1 {
            panic!("width is wider than map")
        }
        if position.y > self.height - 1 {
            panic!("height is larger than map")
        }
        (position.y * self.width + position.x) as usize
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Position {
    x: u32,
    y: u32
}

impl Position {
    pub fn new(x: u32, y: u32) -> Position{
        Position{x: x, y: y}
    }
}

#[derive(Clone)]
enum Direction {
    Up,
    Right,
    Down,
    Left
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(input: &str) -> Result<Direction, Self::Err> {
        match input {
            "up" => Ok(Direction::Up),
            "right" => Ok(Direction::Right),
            "down" => Ok(Direction::Down),
            "left" => Ok(Direction::Left),
            _ => Err(()),
        }
    }
}


impl Universe {  
      pub fn new(map: Map) -> Universe{
        utils::set_panic_hook();

        let player_pos = Position{x: map.width/2, y: map.height/2};
        let player_direction = Direction::Up;

        let asteriods = vec![Asteriod{
            position: Position{x: 45, y: 45},
            radius: 3
        }];

        let mut cells: Vec<Cell> = (0..map.width * map.height)
            .map(|i| {
                if i == player_pos.y * map.width + player_pos.x { 
                    Cell::Active 
                } else {
                    Cell::Inactive
                }
            })
            .collect();
        
        let mut bullets = Vec::with_capacity(100);
        for _ in 0..100 {
            bullets.push(Bullet::new())
        }
        let cur_bullet_index = 0;

        let asteriod_indexes = map.get_asteroid_indexes(&asteriods[0]);
        for i in asteriod_indexes.iter(){
            cells[*i] = Cell::Active;
        }

        Universe {
            cells,

            player_pos,
            player_direction,

            bullets,
            cur_bullet_index,

            asteriods,

            map
        }
    }

    fn get_player_index(&self) -> usize {
        self.map.get_index(self.player_pos.x, self.player_pos.y)
    }

    fn next_bullet_index(&mut self) -> usize {
        let mut next_index = self.cur_bullet_index + 1;
        let bullets_len = self.bullets.len();
        if next_index > bullets_len - 1 {
            next_index = 0;
        }
        self.cur_bullet_index = next_index;
        next_index
    }

    pub fn get_cells(&self) -> &Vec<Cell>{
        &self.cells
    }

    pub fn set_cells(&mut self, cell_locs: &[(u32, u32)]) {
        for cell_loc in cell_locs.iter() {
            let i = self.map.get_index(cell_loc.0, cell_loc.1);
            self.cells[i] = Cell::Active;
        }
    }
    
    pub fn set_player_position(&mut self, position: Position) {
        self.player_pos = position;
    }
    
}

#[wasm_bindgen]
impl Universe {
    pub fn new_default() -> Universe {
        let height = 64;
        let width = 64;
        let map = Map::new(width, height);
        Universe::new(map)
    }


    pub fn width(&self) -> u32 {
        self.map.width
    }

    pub fn height(&self) -> u32 {
        self.map.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    fn render(&mut self) {
        for idx in 0..self.cells.len() {
            self.cells[idx] = Cell::Inactive;
        }

        // TODO: Build a "renderable" trait that describes which indexes an object occupies
        let player_idx = self.map.get_index_from_position(&self.player_pos);
        self.cells[player_idx] = Cell::Active;

        for bullet in self.bullets.iter() {
            if !bullet.active {
                continue;
            }
            let idx = self.map.get_bullet_index(bullet);
            self.cells[idx] = Cell::Active;
        }

        for asteriod in self.asteriods.iter(){
            let asteriod_index = self.map.get_asteroid_indexes(asteriod);
            for idx in asteriod_index {
                self.cells[idx] = Cell::Active;
            }
        }
    }

    pub fn move_player(&mut self, command: String) {
        let old_index = self.get_player_index();

        print!("{}", command);
        self.player_direction = Direction::from_str(&command[..]).unwrap();
        self.player_pos = self.map.position_in_direction(&self.player_pos, &self.player_direction);
        let new_index = self.map.get_index_from_position(&self.player_pos);
    }

    pub fn shoot(&mut self) {
        let i = self.next_bullet_index();
        let bullet = &mut self.bullets[i];

        bullet.direction = self.player_direction.clone();
        bullet.position = self.map.position_in_direction(&self.player_pos, &bullet.direction);
        bullet.active = true;
        let new_index = self.map.get_index_from_position(&bullet.position);
    }
    
    pub fn tick(&mut self) { 
        for bullet in self.bullets.iter_mut() {
            if !bullet.active {
                continue;
            }
            let old_index = self.map.get_bullet_index(bullet);
            self.cells[old_index] = Cell::Inactive;
            let new_bullet_pos = self.map.position_in_direction(&bullet.position, &bullet.direction);
            if new_bullet_pos == bullet.position { // Bullet is at edge of map
                bullet.active = false;
            } else {
                bullet.position = new_bullet_pos;
            }
        }

        self.render()
    }
}
