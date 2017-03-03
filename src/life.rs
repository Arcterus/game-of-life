extern crate graphics;
extern crate piston;

extern crate glutin_window;
extern crate opengl_graphics;

use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;

pub static WINDOW_HEIGHT: usize = 480;
pub static WINDOW_WIDTH: usize = 640;

pub static BLOCK_SIZE: usize = 10;  // NOTE: WINDOW_HEIGHT and WINDOW_WIDTH should be divisible by this

pub enum Neighbor {
   Block(Block),
   Location(Location)
}

pub struct Grid {
   grid: Vec<Vec<Option<Block>>>,
   blocks: Vec<Block>
}

#[derive(Clone, PartialEq, Copy)]
pub struct Block {
   pub loc: Location
}

#[derive(Clone, PartialEq, Copy)]
pub struct Location {
   pub x: usize,
   pub y: usize
}

pub struct App {
   gl: GlGraphics,
   grid: Grid,
   started: bool,
   mouse_loc: (f64, f64)
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
         blocks: vec!()
      }
   }

   pub fn insert(&mut self, block: Block) {
      let (x, y) = (block.loc.x, block.loc.y);
      if !self.valid(x, y) {
         return;
      }
      let gr_loc = &mut self.grid[y][x];
      if *gr_loc == None {
         *gr_loc = Some(block);
         self.blocks.push(gr_loc.unwrap());
      } else if gr_loc.unwrap() != block {
         let idx = {
            let old = gr_loc.unwrap();
            let mut i = 0;
            let len = self.blocks.len();
            while i < len {
               if old == self.blocks[i] {
                  break;
               }
               i += 1;
            }
            i
         };
         *gr_loc = Some(block);
         self.blocks[idx] = gr_loc.unwrap();
      }
   }

   pub fn remove(&mut self, block: &Block) {
      if self.valid(block.loc.x, block.loc.y) {
         let mut i = 0;
         while i < self.blocks.len() {
            if &self.blocks[i] == block {
               self.blocks.remove(i);
               break;
            }
            i += 1;
         }
         self.grid[block.loc.y][block.loc.x] = None;
      }
   }

   pub fn neighbors(&self, block: &Block) -> Vec<Neighbor> {
      let mut vec = vec!();
      if self.valid(block.loc.x, block.loc.y) {
         for i in (if block.loc.x > 0 { -1 } else { 0 })..(if block.loc.x < self.grid.len() - 1 { 2 } else { 1 }) {
            if block.loc.y > 0 {
               let row = &self.grid[block.loc.y - 1];
               let xpos = (i + block.loc.x as i32) as usize;
               vec.push(match row[xpos] {
                  Some(blk) => Neighbor::Block(blk),
                  None => Neighbor::Location(Location::new(xpos, block.loc.y - 1))
               });
            }
            if block.loc.y < self.grid.len() - 1 {
               let row = &self.grid[block.loc.y + 1];
               let xpos = (i + block.loc.x as i32) as usize;
               vec.push(match row[xpos] {
                  Some(blk) => Neighbor::Block(blk),
                  None => Neighbor::Location(Location::new(xpos, block.loc.y + 1))
               });
            }
         }
         let row = &self.grid[block.loc.y];
         if block.loc.x > 0 {
            vec.push(match row[block.loc.x - 1] {
               Some(blk) => Neighbor::Block(blk),
               None => Neighbor::Location(Location::new(block.loc.x - 1, block.loc.y))
            });
         }
         if block.loc.x < self.grid[0].len() - 1 {
            vec.push(match row[block.loc.x + 1] {
               Some(blk) => Neighbor::Block(blk),
               None => Neighbor::Location(Location::new(block.loc.x + 1, block.loc.y))
            });
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
   pub fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
      for block in self.blocks.iter() {
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

      gl.draw(args.viewport(), |c, gl| {
         let block = rectangle::square((self.loc.x * BLOCK_SIZE) as f64,
                                       (self.loc.y * BLOCK_SIZE) as f64,
                                       BLOCK_SIZE as f64);
         rectangle(BLACK, block, c.transform, gl);
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
         mouse_loc: (0.0, 0.0)
      }
   }

   pub fn release(&mut self, button: &Button) {
      match button {
         &Button::Keyboard(key) => self.key_release(key),
		 &Button::Mouse(button) => self.mouse_release(button),
		 _ => { }
	  }
   }

   pub fn key_release(&mut self, key: Key) {
      match key {
         Key::R => {
            self.grid = Grid::new();
            self.started = false;
            self.mouse_loc = (0.0, 0.0);
         }
         Key::P | Key::Return => self.started = !self.started,
         _ => {}
      }
	  println!("released key: {:?}", key);
   }

   pub fn mouse_release(&mut self, button: MouseButton) {
      if !self.started {
         let (mut x, mut y) = self.mouse_loc;
         x = (x - (x as usize % BLOCK_SIZE) as f64) / BLOCK_SIZE as f64;
         y = (y - (y as usize % BLOCK_SIZE) as f64) / BLOCK_SIZE as f64;
         self.grid.insert(Block::new(Location::new(x as usize, y as usize)))
      }
	  println!("release mouse mutton: {:?}", button);
   }

   pub fn mouse_move(&mut self, pos: [f64; 2]) {
      self.mouse_loc = (pos[0], pos[1]);
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
      for block in &self.grid.blocks {
         let live = self.grid.live_neighbors(&block);
         let livelen = live.len();
         if livelen != 2 && livelen != 3 {
            remove.push(block.clone());
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

      self.grid.render(&mut self.gl, args);
   }
}

fn main() {
   assert!(WINDOW_WIDTH % BLOCK_SIZE == 0);
   assert!(WINDOW_HEIGHT % BLOCK_SIZE == 0);

   let opengl = OpenGL::V3_2;

   let mut window: GlutinWindow = WindowSettings::new(
      "Conway's Game of Life",
      [WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32]
   ).fullscreen(false).exit_on_esc(true).build().unwrap();

   let mut app = App::new(GlGraphics::new(opengl));

   let event_settings = EventSettings::new().ups(2).max_fps(60);
   let mut events = Events::new(event_settings);
   while let Some(e) = events.next(&mut window) {
      if let Some(r) = e.render_args() {
         app.render(&r);
	  }

      if let Some(u) = e.update_args() {
         app.update(&u);
	  }

	  if let Some(b) = e.release_args() {
         app.release(&b);
	  }

	  if let Some(c) = e.mouse_cursor_args() {
         app.mouse_move(c);
	  }
   }
}
