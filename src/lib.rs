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

pub trait Renderable {
    fn render(&self) -> Vec<&Position>;
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

    asteriods: Vec<Asteroid>,
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

impl Renderable for Bullet {
    fn render(&self) -> Vec<&Position> {
        if !self.active{
            return vec![]
        }
        vec![&self.position]
    }
}

struct Asteroid {
    positions: Vec<Position>,
}

impl Asteroid {
    fn new(position: Position, diameter: u32) -> Asteroid {
        let mut positions = Vec::new();
        for y in (position.y)..(position.y + diameter) {
            for x in (position.x)..(position.x + diameter) {
                positions.push(Position{x: x, y: y});
            }
        }
        Asteroid{positions: positions}
    }

    fn generate_asteriod_from_position(parent_asteriod: &Asteroid, position: &Position, map: &Map) -> Asteroid{
        let new_asteroid_positions = Asteroid::find_related_positions(vec![*position], vec![*position], parent_asteriod, map);
        log!("Created asteroid with {} squares", new_asteroid_positions.len());
        Asteroid{positions: new_asteroid_positions}
    }

    // TODO: Change from recursion to 'while let'
    fn find_related_positions(mut new_asteroid_positions: Vec<Position>, mut unchecked_positions: Vec<Position>, parent_asteriod: &Asteroid, map: &Map) -> Vec<Position>{
        let position;
        if unchecked_positions.len() == 0 {
            return new_asteroid_positions
        } else {
            // TODO: Handle error
            position = unchecked_positions.pop().unwrap()
        }
        for dir in CARDINAL_DIRECTIONS.iter() {
            let position_in_direction = map.position_in_direction(&position, dir);
            for parent_pos in parent_asteriod.positions.iter(){
                if *parent_pos == position_in_direction {

                    let mut already_checked = false;
                    for checked_pos in new_asteroid_positions.iter() {
                        if *checked_pos == position_in_direction {
                            already_checked = true;
                        }
                    }

                    if !already_checked {
                        new_asteroid_positions.push(position_in_direction);
                        unchecked_positions.push(position_in_direction);
                        break;
                    }
                }
            }
        }
        Asteroid::find_related_positions(new_asteroid_positions, unchecked_positions, parent_asteriod, map)
    }


    // TODO: Map should check collision and pass position to asteriod for removal/asteriod separation
    fn process_collision(&mut self, collision_position: &Position, map: &Map) -> Option<(Asteroid, Asteroid)> {
        // Remove from positions
        let mut collision_position_index: Option<usize> = None;
        for i in 0..self.positions.len(){
            if self.positions[i] == *collision_position {
                collision_position_index = Some(i);
            }
        }
        match collision_position_index {
            None => panic!("could not find collision index"),
            Some(i) => {
                self.positions.remove(i);
                ();
            }
        }
       
        // Check if this collision caused an asteriod separation
        // TODO: Making asteriod position into a map is probably faster/better
        let up = map.position_in_direction(&collision_position, &Direction::Up);
        let right = map.position_in_direction(&collision_position, &Direction::Right);
        let down = map.position_in_direction(&collision_position, &Direction::Down);
        let left = map.position_in_direction(&collision_position, &Direction::Left);
        let mut has_up = false;
        let mut has_right = false;
        let mut has_down = false;
        let mut has_left = false;
        let mut count = 0;
        for pos in self.positions.iter() {
            match pos {
                _ if *pos == up => {
                    has_up = true;
                    count += 1;
                }
                _ if *pos == right => {
                    has_right = true;
                    count += 1;
                }
                _ if *pos == down => {
                    has_down = true;
                    count +=1;
                }
                _ if *pos == left => {
                    has_left = true;
                    count += 1;
                }
                _ => (),
            }
        }
        let mut horizontal = false;
        let mut vertical = false;
        if has_up && has_down {
            vertical = true;
        }
        if has_right && has_left {
            horizontal = true
        }
        let (asteroid_1, asteroid_2): (Asteroid, Asteroid);
        if count == 2 && (vertical && !horizontal) {
            asteroid_1 = Asteroid::generate_asteriod_from_position(self, &up, map);
            asteroid_2 = Asteroid::generate_asteriod_from_position(self, &down, map);
            return Some((asteroid_1, asteroid_2));
        } else if count == 2 && (horizontal && !vertical) {
            asteroid_1 = Asteroid::generate_asteriod_from_position(self, &right, map);
            asteroid_2 = Asteroid::generate_asteriod_from_position(self, &left, map);
            return Some((asteroid_1, asteroid_2));
        }

        return None
    }
}

impl Renderable for Asteroid {
    fn render(&self) -> Vec<&Position> {
        let mut indexes = Vec::new();
        for position in self.positions.iter() {
            indexes.push(position)
        }
        indexes
    }
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

    fn check_collision(&self, obj1: &impl Renderable, obj2: &impl Renderable) -> Option<Position>{
        for obj1_position in obj1.render(){
            for obj2_position in obj2.render(){
                if obj1_position == obj2_position {
                    return Some(*obj1_position)
                }
            }
        }
        None
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
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

const CARDINAL_DIRECTIONS: [Direction; 4] = [Direction::Up, Direction::Right, Direction::Down, Direction::Left];

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

        let asteriods = vec![Asteroid::new(
            Position{x: 45, y: 45},
            9
        )];

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

    fn render(&mut self) {
        for idx in 0..self.cells.len() {
            self.cells[idx] = Cell::Inactive;
        }

        let mut alive_positions = Vec::new();

        alive_positions.push(&self.player_pos);

        // TODO: Track all renderable objects somewhere
        for bullet in self.bullets.iter() {
            let mut bullet_pos = bullet.render();
            alive_positions.append(&mut bullet_pos);
        }

        for asteriod in self.asteriods.iter() {
            let mut asteriod_pos = asteriod.render();
            alive_positions.append(&mut asteriod_pos);
        }

        for pos in alive_positions.iter() {
            let idx = self.map.get_index_from_position(pos);
            self.cells[idx] = Cell::Active;
        }
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


    pub fn move_player(&mut self, command: String) {
        print!("{}", command);
        self.player_direction = Direction::from_str(&command[..]).unwrap();
        self.player_pos = self.map.position_in_direction(&self.player_pos, &self.player_direction);
    }

    pub fn shoot(&mut self) {
        let i = self.next_bullet_index();
        let bullet = &mut self.bullets[i];

        bullet.direction = self.player_direction.clone();
        bullet.position = self.map.position_in_direction(&self.player_pos, &bullet.direction);
        bullet.active = true;
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

            let mut asteroid_removal_index: usize = 0;
            let mut collision_result: Option<(Asteroid, Asteroid)> = None;
            for (i, asteroid) in self.asteriods.iter_mut().enumerate() {
                match self.map.check_collision(bullet, asteroid) {
                    None => continue,
                    Some(collision_position) => {
                        collision_result = asteroid.process_collision(&collision_position, &self.map);
                        bullet.active=false;
                        asteroid_removal_index = i;
                        break;
                    }
                }
            }
            match collision_result{
                Some((a_1, a_2)) => {
                    self.asteriods.remove(asteroid_removal_index);
                    self.asteriods.push(a_1);
                    self.asteriods.push(a_2);
                    log!("{}", self.asteriods.len());
                },
                None => ()
            }
        }

        self.render()
    }
}
