extern crate graphics;
extern crate piston;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use graphics::*;
use graphics::math::Matrix2d;
use opengl_graphics::{GlGraphics, OpenGL};
use glutin_window::GlutinWindow as Window;

pub const WINDOW_HEIGHT: u32 = 480;
pub const WINDOW_WIDTH: u32 = 640;

pub const BLOCK_SIZE: u32 = 10;  // NOTE: WINDOW_HEIGHT and WINDOW_WIDTH should be divisible by this

pub const GRID_WIDTH: usize = (WINDOW_WIDTH / BLOCK_SIZE) as usize;
pub const GRID_HEIGHT: usize = (WINDOW_HEIGHT / BLOCK_SIZE) as usize;

pub enum Neighbor {
	Block(Block),
	Location(Location)
}

pub struct Grid {
	grid: Vec<Vec<Option<Block>>>,
	blocks: Vec<Block>
}

#[derive(Copy, Clone, PartialEq)]
pub struct Block {
	pub loc: Location
}

#[derive(Copy, Clone, PartialEq)]
pub struct Location {
	pub x: usize,
	pub y: usize
}

pub const FRAME_DURATION: f64 = 0.10; // seconds

pub struct App {
	gl: GlGraphics,
	grid: Grid,
	started: bool,
	mouse_loc: (f64, f64),
	t: f64  // seconds since last frame
}

impl Grid {
	pub fn new() -> Grid {
		let mut rows: Vec<Vec<Option<Block>>> = vec!();
		rows.reserve(GRID_HEIGHT);
		for _ in 0 .. GRID_HEIGHT {
			rows.push(vec![None; GRID_WIDTH]);
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
		match self.grid[y][x] {
			None => {
				self.grid[y][x] = Some(block);
				self.blocks.push(block);
			},
			Some(old) => {
				if old != block {
					let idx = {
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
					self.grid[y][x] = Some(block);
					self.blocks[idx] = block;
				}
			}
		}
	}

	pub fn remove(&mut self, block: &Block) {
		if self.valid(block.loc.x, block.loc.y) {
			let mut i = 0;
			while i < self.blocks.len() {
				if self.blocks[i] == *block {
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
			for i in (if block.loc.x > 0 { -1 } else { 0 })
			         ..
			         (if block.loc.x < GRID_WIDTH - 1 { 2 } else { 1 }) {
				if block.loc.y > 0 {
					let row = &self.grid[block.loc.y - 1];
					let xpos = (i + block.loc.x as isize) as usize;
					vec.push(match row[xpos] {
						Some(blk) => Neighbor::Block(blk),
						None => Neighbor::Location(Location::new(xpos, block.loc.y - 1))
					});
				}
				if block.loc.y < self.grid.len() - 1 {
					let row = &self.grid[block.loc.y + 1];
					let xpos = (i + block.loc.x as isize) as usize;
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
	pub fn render(&self, t: &Matrix2d, gl: &mut GlGraphics) {
		for block in self.blocks.iter() {
			block.render(t, gl);
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
	pub fn render(&self, t: &Matrix2d, gl: &mut GlGraphics) {
		const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
		let coords = [self.loc.x as f64, self.loc.y as f64, 1.0, 1.0];
		rectangle(BLACK, coords, *t, gl);
	}
}

impl Location {
	#[inline]
	pub fn new(x: usize, y: usize) -> Location {
		assert!(x <= GRID_WIDTH);
		assert!(y <= GRID_HEIGHT);
		Location {
			x: x,
			y: y
		}
	}
}

impl App {
	pub fn new(gl: GlGraphics) -> App {
		App {
			gl: gl,
			grid: Grid::new(),
			started: false,
			mouse_loc: (0.0, 0.0),
			t: 0.0
		}
	}

	fn update(&mut self, args: &UpdateArgs) {
		self.t += args.dt;
		while self.t > FRAME_DURATION {
			self.update_one_frame();
			self.t -= FRAME_DURATION;
		}
	}

	fn update_one_frame(&mut self) {
		let mut remove = vec![];
		let mut add = vec![];

		for block in self.grid.blocks.iter() {
			let live = self.grid.live_neighbors(block);
			let livelen = live.len();
			if livelen != 2 && livelen != 3 {
				remove.push(block.clone());
			}
			for &loc in self.grid.dead_neighbors(block).iter() {
				let block = Block::new(loc);
				if self.grid.live_neighbors(&block).len() == 3 {
					add.push(block);
				}
			}
		}
		for block in remove.iter() {
			self.grid.remove(block);
		}
		for block in add {
			self.grid.insert(block);
		}
	}

	fn key_release(&mut self, key: Key) {
		match key {
			Key::R => {
				self.grid = Grid::new();
				self.started = false;
				self.mouse_loc = (0.0, 0.0);
			}
			Key::P | Key::Return | Key::Space =>
				self.started = !self.started,
			_ => {}
		}
		println!("released key: {:?}", key);
	}

	fn mouse_release(&mut self, btn: MouseButton) {
		if !self.started {
			let (x, y) = self.mouse_loc;
			let x = (x - (x as u32 % BLOCK_SIZE) as f64) / BLOCK_SIZE as f64;
			let y = (y - (y as u32 % BLOCK_SIZE) as f64) / BLOCK_SIZE as f64;
			self.grid.insert(Block::new(Location::new(x as usize, y as usize)))
		}
		println!("released mouse button: {:?}", btn);
	}

	fn mouse_move(&mut self, x: f64, y: f64) {
		self.mouse_loc = (x, y);
	}

	fn render(&mut self, args: &RenderArgs) {
		const WHITE:  [f32; 4] = [1.0, 1.0, 1.0, 1.0];

		let grid = &self.grid;
		self.gl.draw(args.viewport(), |c, gl| {
			graphics::clear(WHITE, gl);
			grid.render(&c.transform.scale(BLOCK_SIZE as f64, BLOCK_SIZE as f64), gl);
		});
	}
}

fn main() {
	// Change this to OpenGL::V2_1 if not working.
	let opengl = OpenGL::V3_2;

	assert!(WINDOW_WIDTH % BLOCK_SIZE == 0);
	assert!(WINDOW_HEIGHT % BLOCK_SIZE == 0);
	let window: Window = WindowSettings::new(
			"Conway's Game of Life".to_string(),
			[WINDOW_WIDTH, WINDOW_HEIGHT]
		)
		.opengl(opengl)
		.exit_on_esc(true)
		.build()
		.unwrap();

	let mut app = App::new(GlGraphics::new(opengl));
	for e in window.events() {
		match e {
			Event::Render(ref r) =>
				app.render(r),
			Event::Update(ref u) =>
				if app.started {
					app.update(u)
				},
			Event::Input(Input::Release(Button::Keyboard(key))) =>
				app.key_release(key),
			Event::Input(Input::Release(Button::Mouse(btn))) =>
				app.mouse_release(btn),
			Event::Input(Input::Move(Motion::MouseCursor(x, y))) =>
				app.mouse_move(x, y),
			_ => {}
		}
	}
}
