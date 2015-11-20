#[macro_use]
extern crate clap;

extern crate graphics;
extern crate piston;

extern crate glutin_window;
extern crate opengl_graphics;

use std::str::FromStr;
use std::collections::HashSet;
use clap::Arg;
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use piston::window::AdvancedWindow;
use glutin_window::GlutinWindow;

pub const WINDOW_TITLE: &'static str = "Conway's Game of Life";

pub const WINDOW_HEIGHT: usize = 480;
pub const WINDOW_WIDTH: usize = 640;

pub const DEFAULT_SPEED: u64 = 2;

pub const BLOCK_SIZE: usize = 10;  // NOTE: WINDOW_HEIGHT and WINDOW_WIDTH should be divisible by this

pub enum Neighbor {
   Block(Block),
   Location(Location)
}

pub struct Grid {
   grid: Vec<Vec<Option<Block>>>,
   blocks: HashSet<Block>
}

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub struct Block {
   pub loc: Location
}

#[derive(Clone, PartialEq, Eq, Hash, Copy)]
pub struct Location {
   pub x: usize,
   pub y: usize
}

pub struct App {
   gl: GlGraphics,
   grid: Grid,
   started: bool,
   mouse_loc: (f64, f64),
   mouse_down: (bool, bool)   // (left, right)
}

impl Grid {
   pub fn new() -> Grid {
      let mut rows: Vec<Vec<Option<Block>>> = vec!();
      rows.reserve(WINDOW_HEIGHT / BLOCK_SIZE);
      for _ in 0..(WINDOW_HEIGHT / BLOCK_SIZE) {
         rows.push(vec![None; WINDOW_WIDTH / BLOCK_SIZE]);
      }
      Grid {
         grid: rows,
         blocks: HashSet::new()
      }
   }

   pub fn insert(&mut self, block: Block) {
      let (x, y) = (block.loc.x, block.loc.y);
      if self.valid(x, y) {
         if self.grid[y][x] == None {
            self.grid[y][x] = Some(block);
            self.blocks.insert(self.grid[y][x].unwrap());
         } else if self.grid[y][x].unwrap() != block {
            // FIXME: is this even necessary?
            self.blocks.remove(&self.grid[y][x].unwrap());
            self.grid[y][x] = Some(block);
            self.blocks.insert(self.grid[y][x].unwrap());
         }
      }
   }

   pub fn remove(&mut self, block: &Block) {
      if self.valid(block.loc.x, block.loc.y) {
         self.blocks.remove(block);
         self.grid[block.loc.y][block.loc.x] = None;
      }
   }

   pub fn neighbors(&self, block: &Block) -> Vec<Neighbor> {
      let mut vec = vec!();
      if self.valid(block.loc.x, block.loc.y) {
         let places = [(-1, -1), (-1, 0), (-1, 1), (0, 1), (0, -1), (1, 1), (1, -1), (1, 0)];
         for &(x, y) in &places {
            let xpos = (block.loc.x as isize + x) as usize;
            let ypos = (block.loc.y as isize + y) as usize;
            if self.valid(xpos, ypos) {
               let row = &self.grid[ypos];
               vec.push(match row[xpos] {
                  Some(blk) => Neighbor::Block(blk),
                  None => Neighbor::Location(Location::new(xpos, ypos))
               });
            }
         }
      }
      vec
   }

   #[inline]
   pub fn live_neighbors(&self, block: &Block) -> Vec<Block> {
      let mut live = vec!();
      for neighbor in self.neighbors(block) {
         match neighbor {
            Neighbor::Block(blk) => live.push(blk),
            _ => {}
         }
      }
      live
   }

   #[inline]
   pub fn dead_neighbors(&self, block: &Block) -> Vec<Location> {
      let mut dead = vec!();
      for neighbor in self.neighbors(block) {
         match neighbor {
            Neighbor::Location(loc) => dead.push(loc),
            _ => {}
         }
      }
      dead
   }

   pub fn contains(&self, block: &Block) -> bool {
      if self.valid(block.loc.x, block.loc.y) {
         self.grid[block.loc.y][block.loc.x].is_some()
      } else {
         false
      }
   }

   #[inline]
   fn valid(&self, x: usize, y: usize) -> bool {
      y < self.grid.len() && x < self.grid[0].len()
   }

   #[inline]
   pub fn render(&self, gl: &mut GlGraphics, started: bool, args: &RenderArgs) {
      use graphics::grid::*;
      use graphics::line::Line;

      const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

      if !started {
         gl.draw(args.viewport(), |c, gl| {
            Grid {
               cols: self.grid[0].len() as u32,
               rows: self.grid.len() as u32,
               units: BLOCK_SIZE as f64
            }.draw(&Line::new(BLACK, 1.0), &c.draw_state, c.transform, gl);
         });
      }

      for block in &self.blocks {
         block.render(gl, args);
      }
   }
}

impl Block {
   #[inline]
   pub fn new(loc: Location) -> Block {
      Block {
         loc: loc
      }
   }

   #[inline]
   pub fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
      use graphics::*;

      const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

      let block = rectangle::square(0.0, 0.0, BLOCK_SIZE as f64);

      gl.draw(args.viewport(), |c, gl| {
         rectangle(BLACK,
                   block,
                   c.transform.trans((self.loc.x * BLOCK_SIZE) as f64,
                                     (self.loc.y * BLOCK_SIZE) as f64),
                   gl);
      });
   }
}

impl Location {
   #[inline]
   pub fn new(x: usize, y: usize) -> Location {
      assert!(x <= WINDOW_WIDTH / BLOCK_SIZE);
      assert!(y <= WINDOW_HEIGHT / BLOCK_SIZE);
      Location {
         x: x,
         y: y
      }
   }
}

impl App {
   #[inline]
   pub fn new(gl: GlGraphics) -> App {
      App {
         gl: gl,
         grid: Grid::new(),
         started: false,
         mouse_loc: (0.0, 0.0),
         mouse_down: (false, false)
      }
   }

   pub fn press(&mut self, button: &Button) {
      match button {
         &Button::Mouse(button) => self.mouse_press(button),
         _ => {}
      }
   }

   pub fn release<W: AdvancedWindow>(&mut self, window: &mut W, button: &Button) {
      match button {
         &Button::Keyboard(key) => self.key_release(window, key),
         &Button::Mouse(button) => self.mouse_release(button),
         _ => { }
      }
   }

   pub fn key_release<W: AdvancedWindow>(&mut self, window: &mut W, key: Key) {
      match key {
         Key::R => {
            self.grid = Grid::new();
            self.started = false;
            self.mouse_loc = (0.0, 0.0);
         }
         Key::P | Key::Return => {
            if self.started {
               self.started = false;
               window.set_title(format!("{} (paused)", WINDOW_TITLE));
            } else {
               self.started = true;
               window.set_title(WINDOW_TITLE.to_string());
            }
         }
         _ => {}
      }
      println!("released key: {:?}", key);
   }

   fn mouse_press(&mut self, button: MouseButton) {
      let (left, right) = self.mouse_down;
      self.mouse_down = match button {
         MouseButton::Left => (true, false),
         MouseButton::Right => (false, true),
         _ => (left, right)
      };
      self.mouse_paint();
      println!("press mouse button: {:?}", button);
   }

   fn mouse_release(&mut self, button: MouseButton) {
      let (left, right) = self.mouse_down;
      self.mouse_down = match button {
         MouseButton::Left => (false, right),
         MouseButton::Right => (left, false),
         _ => (left, right)
      };
      println!("release mouse mutton: {:?}", button);
   }

   pub fn mouse_move(&mut self, pos: [f64; 2]) {
      self.mouse_loc = (pos[0], pos[1]);
      self.mouse_paint();
   }

   fn mouse_paint(&mut self) {
      let (left, right) = self.mouse_down;
      if !self.started && (left || right) {
         let (mut x, mut y) = self.mouse_loc;
         x = (x - (x as usize % BLOCK_SIZE) as f64) / BLOCK_SIZE as f64;
         y = (y - (y as usize % BLOCK_SIZE) as f64) / BLOCK_SIZE as f64;
         let block = Block::new(Location::new(x as usize, y as usize));
         if left {
            self.grid.insert(block);
         } else {
            self.grid.remove(&block);
         }
      }
   }

   #[cfg(random)]
   #[inline]
   fn update_logic(&mut self) {
      use std::rand::random;

      let mut x = (random::<f64>() * WINDOW_WIDTH as f64) as u32;
      x = (x - x % BLOCK_SIZE) / BLOCK_SIZE;
      let mut y = (random::<f64>() * WINDOW_HEIGHT as f64) as u32;
      y = (y - y % BLOCK_SIZE) / BLOCK_SIZE;
      self.grid.insert(Block::new(Location::new(x, y)));
   }

   #[cfg(not(random))]
   #[inline]
   fn update_logic(&mut self) {
      let mut remove = vec!();
      let mut add = vec!();
      
      for &block in &self.grid.blocks {
         let live = self.grid.live_neighbors(&block);
         let livelen = live.len();
         if livelen != 2 && livelen != 3 {
            remove.push(block);
         }
         for loc in self.grid.dead_neighbors(&block) {
            let block = Block::new(loc);
            if self.grid.live_neighbors(&block).len() == 3 {
               add.push(block);
            }
         }
      }
      for block in remove {
         self.grid.remove(&block);
      }
      for block in add {
         self.grid.insert(block);
      }
   }

   pub fn update(&mut self, _: &UpdateArgs) {
      if self.started {
         self.update_logic();
      }
   }

   pub fn render(&mut self, args: &RenderArgs) {
      (&mut self.gl).viewport(0, 0, args.width as i32, args.height as i32);
      self.gl.draw(args.viewport(), |_, gl| {
         clear([1.0, 1.0, 1.0, 1.0], gl);
      });

      self.grid.render(&mut self.gl, self.started, args);
   }
}

fn main() {
   assert!(WINDOW_WIDTH % BLOCK_SIZE == 0);
   assert!(WINDOW_HEIGHT % BLOCK_SIZE == 0);

   let matches = clap::App::new("game-of-life")
                               .version(crate_version!())
                               .author(crate_authors!())
                               .about(crate_description!())
                               .arg(Arg::with_name("speed")
                                    .short("s")
                                    .long("speed")
                                    .value_name("SPEED")
                                    .help("Sets the speed of each update")
                                    .takes_value(true))
                               .get_matches();

   // TODO: move error into clap parsing
   let speed = if let Some(valstr) = matches.value_of("speed") {
      match u64::from_str(valstr) {
         Ok(val) => val,
         Err(_) => {
            println!("error: {} is not a valid speed", valstr);
            DEFAULT_SPEED
         }
      }
   } else {
      DEFAULT_SPEED
   };

   let opengl = OpenGL::V3_2;

   let mut window: GlutinWindow = WindowSettings::new(
      WINDOW_TITLE,
      [WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32]
   ).fullscreen(false).exit_on_esc(true).build().unwrap();

   let mut app = App::new(GlGraphics::new(opengl));

   let event_settings = EventSettings::new().ups(speed).max_fps(60);
   let mut events = Events::new(event_settings);
   while let Some(e) = events.next(&mut window) {
      if let Some(r) = e.render_args() {
         app.render(&r);
      }

      if let Some(u) = e.update_args() {
         app.update(&u);
      }

      if let Some(b) = e.release_args() {
         app.release(&mut window, &b);
      }

      if let Some(b) = e.press_args() {
         app.press(&b);
      }

      if let Some(c) = e.mouse_cursor_args() {
         app.mouse_move(c);
      }
   }
}
