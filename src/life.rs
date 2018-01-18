#![feature(conservative_impl_trait)]

#[macro_use]
extern crate clap;
extern crate fnv;
extern crate graphics;
extern crate piston;
extern crate fps_counter;
extern crate glutin_window;
extern crate opengl_graphics;

use fnv::FnvHashSet as HashSet;
use clap::Arg;
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use piston::window::AdvancedWindow;
use glutin_window::GlutinWindow;
use fps_counter::FPSCounter;

pub const WINDOW_TITLE: &'static str = "Conway's Game of Life";

pub const DEFAULT_WINDOW_HEIGHT: u32 = 480;
pub const DEFAULT_WINDOW_WIDTH: u32 = 640;
pub const DEFAULT_SPEED: u64 = 2;

pub const BLOCK_SIZE: usize = 10;  // NOTE: window height and width should be divisible by this

pub enum Neighbor {
   Block(Block),
   Location(Location)
}

pub struct Grid {
   width: usize,
   height: usize,
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
   fps: FPSCounter,
   width: u32,
   height: u32,
   gl: GlGraphics,
   grid: Grid,
   started: bool,
   zoom: f64,
   mouse_loc: (f64, f64),
   mouse_down: (bool, bool)   // (left, right)
}

impl Grid {
   pub fn new(width: usize, height: usize) -> Grid {
      let mut rows: Vec<Vec<Option<Block>>> = vec!();
      rows.reserve(height / BLOCK_SIZE);
      for _ in 0..(height / BLOCK_SIZE) {
         rows.push(vec![None; width / BLOCK_SIZE]);
      }
      Grid {
         width: width,
         height: height,
         grid: rows,
         blocks: HashSet::default()
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

   pub fn neighbors<'a>(&'a self, block: &'a Block) -> Option<impl Iterator<Item = Neighbor> + 'a> {
      if self.valid(block.loc.x, block.loc.y) {
         const PLACES: [(isize, isize); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, 1), (0, -1), (1, 1), (1, -1), (1, 0)];
         
         Some(PLACES.into_iter().filter_map(move |&(x, y)| {
            let xpos = (block.loc.x as isize + x) as usize;
            let ypos = (block.loc.y as isize + y) as usize;
            if self.valid(xpos, ypos) {
               Some(match self.grid[ypos][xpos] {
                  Some(blk) => Neighbor::Block(blk),
                  None => Neighbor::Location(Location::new(xpos, ypos))
               })
            } else {
               None
            }
         }))
      } else {
         None
      }
   }

   #[inline]
   pub fn live_neighbors<'a>(&'a self, block: &'a Block) -> impl Iterator<Item = Block> + 'a {
      self.neighbors(block).map(|neighbors| {
         neighbors.into_iter().filter_map(|neighbor| {
            match neighbor {
               Neighbor::Block(blk) => Some(blk),
               _ => None
            }
         })
      }).into_iter().flat_map(|elem| elem)
   }

   #[inline]
   pub fn dead_neighbors<'a>(&'a self, block: &'a Block) -> impl Iterator<Item = Location> + 'a {
      self.neighbors(block).map(|neighbors| {
         neighbors.into_iter().filter_map(|neighbor| {
            match neighbor {
               Neighbor::Location(loc) => Some(loc),
               _ => None
            }
         })
      }).into_iter().flat_map(|elem| elem)
   }

   #[inline]
   pub fn contains(&self, block: &Block) -> bool {
      self.valid(block.loc.x, block.loc.y) && self.grid[block.loc.y][block.loc.x].is_some()
   }

   #[inline]
   fn valid(&self, x: usize, y: usize) -> bool {
      y < self.grid.len() && x < self.grid[0].len()
   }

   #[inline]
   pub fn render(&mut self, gl: &mut GlGraphics, started: bool, zoom_factor: f64, args: &RenderArgs) {
      use graphics::grid::*;
      use graphics::line::Line;

      const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

      // FIXME: embed the width stuff in the height section
      if self.height as f64 / zoom_factor > self.grid.len() as f64 {
         let len = self.grid[0].len();
         for _ in 0..((self.height as f64 * zoom_factor).round() as u64) {
            self.grid.push(vec![None; len]);
         }
      }
      if self.width as f64 / zoom_factor > self.grid[0].len() as f64 {
         let extension = vec![None; (self.width as f64 * zoom_factor).round() as usize];
         for row in &mut self.grid {
            row.extend(extension.iter())
         }
      }

      if !started {
         gl.draw(args.viewport(), |c, gl| {
            Grid {
               cols: (self.grid[0].len() as f64 * zoom_factor) as u32,
               rows: (self.grid.len() as f64 * zoom_factor) as u32,
               units: (BLOCK_SIZE as f64) * zoom_factor
            }.draw(&Line::new(BLACK, 1.0), &c.draw_state, c.transform, gl);
         });
      }

      for block in &self.blocks {
         block.render(gl, zoom_factor, args);
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
   pub fn render(&self, gl: &mut GlGraphics, zoom_factor: f64, args: &RenderArgs) {
      use graphics::*;

      const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

      let block = rectangle::square(0.0, 0.0, BLOCK_SIZE as f64 * zoom_factor);

      gl.draw(args.viewport(), |c, gl| {
         rectangle(BLACK,
                   block,
                   c.transform.trans((self.loc.x * BLOCK_SIZE) as f64 * zoom_factor,
                                     (self.loc.y * BLOCK_SIZE) as f64 * zoom_factor),
                   gl);
      });
   }
}

impl Location {
   #[inline]
   pub fn new(x: usize, y: usize) -> Location {
      Location {
         x: x,
         y: y
      }
   }
}

impl App {
   #[inline]
   pub fn new(width: u32, height: u32, gl: GlGraphics) -> App {
      App {
         fps: FPSCounter::new(),
         width: width,
         height: height,
         gl: gl,
         grid: Grid::new(width as usize, height as usize),
         started: false,
         zoom: 0.0,
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
            self.grid = Grid::new(self.width as usize, self.height as usize);
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

   pub fn mouse_scroll(&mut self, pos: [f64; 2]) {
      self.zoom += pos[1];
   }

   fn mouse_paint(&mut self) {
      let (left, right) = self.mouse_down;
      if !self.started && (left || right) {
         let (mut x, mut y) = self.mouse_loc;
         x = (x - (x as usize % ((BLOCK_SIZE as f64 * self.zoom_factor())) as usize) as f64) / (BLOCK_SIZE as f64 * self.zoom_factor());
         y = (y - (y as usize % ((BLOCK_SIZE as f64 * self.zoom_factor())) as usize) as f64) / (BLOCK_SIZE as f64 * self.zoom_factor());

         let block = Block::new(Location::new(x as usize, y as usize));
         if left {
            self.grid.insert(block);
         } else {
            self.grid.remove(&block);
         }
      }
   }

   // FIXME: this is now out of date due to zooming
   #[cfg(random)]
   #[inline]
   fn update_logic(&mut self) {
      use std::rand::random;

      let mut x = (random() * self.width as f64) as u32;
      x = (x - x % BLOCK_SIZE) / BLOCK_SIZE;
      let mut y = (random() * self.height as f64) as u32;
      y = (y - y % BLOCK_SIZE) / BLOCK_SIZE;
      self.grid.insert(Block::new(Location::new(x, y)));
   }

   #[cfg(not(random))]
   #[inline]
   fn update_logic(&mut self) {
      let mut remove = vec!();
      let mut add = vec!();

      for &block in &self.grid.blocks {
         let live = self.grid.live_neighbors(&block).count();
         if live != 2 && live != 3 {
            remove.push(block);
         }
         add.extend(self.grid.dead_neighbors(&block).filter_map(|loc| {
            let block = Block::new(loc);
            if self.grid.live_neighbors(&block).count() == 3 {
               Some(block)
            } else {
               None
            }
         }));
      }
      for block in remove {
         self.grid.remove(&block);
      }
      for block in add {
         self.grid.insert(block);
      }
   }

   fn zoom_factor(&self) -> f64 {
      if self.zoom < 0.0 {
         1.0 / (-self.zoom + 1.0)
      } else {
         self.zoom + 1.0
      }
   }

   pub fn update(&mut self, _: &UpdateArgs) {
      if self.started {
         self.update_logic();
      }
   }

   pub fn render(&mut self, args: &RenderArgs) {
      println!("{}", self.fps.tick());

      self.gl.viewport(0, 0, args.width as i32, args.height as i32);
      self.gl.draw(args.viewport(), |_, gl| {
         clear([1.0, 1.0, 1.0, 1.0], gl);
      });

      let factor = self.zoom_factor();
      self.grid.render(&mut self.gl, self.started, factor, args);
   }
}

fn main() {
   assert!(DEFAULT_WINDOW_WIDTH % BLOCK_SIZE as u32 == 0);
   assert!(DEFAULT_WINDOW_HEIGHT % BLOCK_SIZE as u32 == 0);

   let matches = clap::App::new("game-of-life")
                               .version(crate_version!())
                               .author(crate_authors!())
                               .about(crate_description!())
                               .arg(Arg::with_name("speed")
                                    .short("s")
                                    .long("speed")
                                    .value_name("SPEED")
                                    .help("Sets the speed of each update")
                                    .takes_value(true)
                                    .validator(is_positive))
                               .arg(Arg::with_name("width")
                                    .short("w")
                                    .long("width")
                                    .value_name("WIDTH")
                                    .help("Sets the width of the window")
                                    .takes_value(true)
                                    .validator(is_valid_width_or_height))
                               .arg(Arg::with_name("height")
                                    .short("h")
                                    .long("height")
                                    .value_name("HEIGHT")
                                    .help("Sets the height of the window")
                                    .takes_value(true)
                                    .validator(is_valid_width_or_height))
                               // FIXME: set up so the value is used
                               .arg(Arg::with_name("block")
                                    .short("b")
                                    .long("block")
                                    .value_name("SIZE")
                                    .help("Sets the size of the blocks")
                                    .takes_value(true)
                                    .validator(is_positive))
                               .get_matches();

   let speed = matches.value_of("speed").map(|valstr| valstr.parse::<u64>().unwrap())
                                        .unwrap_or(DEFAULT_SPEED);
   let width = matches.value_of("width").map(|valstr| valstr.parse::<u32>().unwrap())
                                        .unwrap_or(DEFAULT_WINDOW_WIDTH);
   let height = matches.value_of("height").map(|valstr| valstr.parse::<u32>().unwrap())
                                          .unwrap_or(DEFAULT_WINDOW_HEIGHT);

   // TODO: switch to gfx-rs backend
   let opengl = OpenGL::V3_2;

   let mut window: GlutinWindow = WindowSettings::new(
      WINDOW_TITLE,
      [width as u32, height as u32]
   ).fullscreen(false).exit_on_esc(true).build().unwrap();

   let mut app = App::new(width, height, GlGraphics::new(opengl));

   let event_settings = EventSettings::new().ups(speed);
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

      if let Some(s) = e.mouse_scroll_args() {
         app.mouse_scroll(s);
      }
   }
}

fn is_valid_width_or_height(valstr: String) -> Result<(), String> {
   if let Ok(val) = valstr.parse::<u32>() {
      if val % BLOCK_SIZE as u32 == 0 {
         return Ok(())
      }
   }

   Err(format!("{} is not a valid size (must be positive and divisible by {}", valstr, BLOCK_SIZE))
}

fn is_positive(valstr: String) -> Result<(), String> {
   if valstr.parse::<u32>().is_ok() {
      Ok(())
   } else {
      Err(format!("{} is not a valid number (must be positive)", valstr))
   }
}
