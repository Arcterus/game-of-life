#![feature(globs, phase)]

extern crate collections;
#[phase(plugin, link)] extern crate log;
extern crate graphics;
extern crate piston;

use graphics::*;
use piston::{Game, GameIteratorSettings, GameWindowSDL2, GameWindowSettings, KeyReleaseArgs, MouseMoveArgs, MouseReleaseArgs, RenderArgs};

pub static WINDOW_HEIGHT: uint = 480;
pub static WINDOW_WIDTH: uint = 640;

pub static BLOCK_SIZE: uint = 10;  // NOTE: WINDOW_HEIGHT and WINDOW_WIDTH should be divisible by this

pub enum Neighbor {
	Block(Block),
	Location(Location)
}

pub struct Grid {
	grid: Vec<Vec<Option<Block>>>,
	blocks: Vec<Block>
}

#[deriving(Clone, PartialEq)]
pub struct Block {
	pub loc: Location
}

#[deriving(Clone, PartialEq)]
pub struct Location {
	pub x: uint,
	pub y: uint
}

pub struct App {
	gl: Gl,
	grid: Grid,
	started: bool,
	count: uint,
	mouse_loc: (f64, f64)
}

impl Grid {
	pub fn new() -> Grid {
		let mut rows: Vec<Vec<Option<Block>>> = vec!();
		rows.reserve(WINDOW_HEIGHT / BLOCK_SIZE);
		for _ in range(0, WINDOW_HEIGHT / BLOCK_SIZE) {
			rows.push(Vec::from_elem(WINDOW_WIDTH / BLOCK_SIZE, None));
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
		let gr_loc = self.grid.get_mut(y).get_mut(x);
		if *gr_loc == None {
			*gr_loc = Some(block);
			self.blocks.push(gr_loc.unwrap());
		} else if gr_loc.unwrap() != block {
			let idx = {
				let old = gr_loc.get_ref();
				let mut i = 0;
				let len = self.blocks.len();
				while i < len {
					if old == self.blocks.get(i) {
						break;
					}
					i += 1;
				}
				i
			};
			*gr_loc = Some(block);
			*self.blocks.get_mut(idx) = gr_loc.unwrap();
		}
	}

	pub fn remove(&mut self, block: &Block) {
		if self.valid(block.loc.x, block.loc.y) {
			let mut i = 0;
			while i < self.blocks.len() {
				if self.blocks.get(i) == block {
					self.blocks.remove(i);
					break;
				}
				i += 1;
			}
			let gr_loc = self.grid.get_mut(block.loc.y).get_mut(block.loc.x);
			*gr_loc = None;
		}
	}

	pub fn neighbors(&self, block: &Block) -> Vec<Neighbor> {
		let mut vec = vec!();
		if self.valid(block.loc.x, block.loc.y) {
			for i in range(if block.loc.x > 0 { -1 } else { 0 }, if block.loc.x < self.grid.len() - 1 { 2 } else { 1 }) {
				if block.loc.y > 0 {
					let row = self.grid.get(block.loc.y - 1);
					let xpos = (i + block.loc.x as int) as uint;
					vec.push(match *row.get(xpos) {
						Some(blk) => Block(blk),
						None => Location(Location::new(xpos, block.loc.y - 1))
					});
				}
				if block.loc.y < self.grid.len() - 1 {
					let row = self.grid.get(block.loc.y + 1);
					let xpos = (i + block.loc.x as int) as uint;
					vec.push(match *row.get(xpos) {
						Some(blk) => Block(blk),
						None => Location(Location::new(xpos, block.loc.y + 1))
					});
				}
			}
			let row = self.grid.get(block.loc.y);
			if block.loc.x > 0 {
				vec.push(match *row.get(block.loc.x - 1) {
					Some(blk) => Block(blk),
					None => Location(Location::new(block.loc.x - 1, block.loc.y))
				});
			}
			if block.loc.x < self.grid.get(0).len() - 1 {
				vec.push(match *row.get(block.loc.x + 1) {
					Some(blk) => Block(blk),
					None => Location(Location::new(block.loc.x + 1, block.loc.y))
				});
			}
		}
		vec
	}

	#[inline]
	pub fn live_neighbors(&self, block: &Block) -> Vec<Block> {
		let mut live = vec!();
		for neighbor in self.neighbors(block).move_iter() {
			match neighbor {
				Block(blk) => live.push(blk),
				_ => {}
			}
		}
		live
	}

	#[inline]
	pub fn dead_neighbors(&self, block: &Block) -> Vec<Location> {
		let mut dead = vec!();
		for neighbor in self.neighbors(block).move_iter() {
			match neighbor {
				Location(loc) => dead.push(loc),
				_ => {}
			}
		}
		dead
	}

	pub fn contains(&self, block: &Block) -> bool {
		if self.valid(block.loc.x, block.loc.y) {
			self.grid.get(block.loc.y).get(block.loc.x).is_some()
		} else {
			false
		}
	}

	#[inline]
	fn valid(&self, x: uint, y: uint) -> bool {
		y < self.grid.len() && x < self.grid.get(0).len()
	}

	#[inline]
	pub fn render(&self, gl: &mut Gl, win_ctx: &Context) {
		for block in self.blocks.iter() {
			block.render(gl, win_ctx);
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
	pub fn render(&self, gl: &mut Gl, win_ctx: &Context) {
		win_ctx
		       .rect((self.loc.x * BLOCK_SIZE) as f64, (self.loc.y * BLOCK_SIZE) as f64, BLOCK_SIZE as f64, BLOCK_SIZE as f64)
		       .rgb(0.0, 0.0, 0.0).fill(gl);
	}
}

impl Location {
	#[inline]
	pub fn new(x: uint, y: uint) -> Location {
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
	pub fn new() -> App {
		App {
			gl: Gl::new(),
			grid: Grid::new(),
			started: false,
			count: 30,
			mouse_loc: (0.0, 0.0)
		}
	}

	#[cfg(random)]
	#[inline]
	fn render_logic(&mut self) {
		use std::rand::random;

		let mut x = (random::<f64>() * WINDOW_WIDTH as f64) as uint;
		x = (x - x % BLOCK_SIZE) / BLOCK_SIZE;
		let mut y = (random::<f64>() * WINDOW_HEIGHT as f64) as uint;
		y = (y - y % BLOCK_SIZE) / BLOCK_SIZE;
		self.grid.insert(Block::new(Location::new(x, y)));
	}

	#[cfg(not(random))]
	#[inline]
	fn render_logic(&mut self) {
		let mut remove = vec!();
		let mut add = vec!();
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
		for block in add.move_iter() {
			self.grid.insert(block);
		}
	}
}

impl Game for App {
	fn key_release(&mut self, args: &KeyReleaseArgs) {
		match args.key {
			piston::keyboard::R => {
				self.grid = Grid::new();
				self.started = false;
				self.count = 30;
				self.mouse_loc = (0.0, 0.0);
			}
			piston::keyboard::P | piston::keyboard::Return => self.started = !self.started,
			_ => {}
		}
		debug!("released key: {}", args.key);
	}

	fn mouse_release(&mut self, args: &MouseReleaseArgs) {
		if !self.started {
			let (mut x, mut y) = self.mouse_loc;
			x = (x - (x as uint % BLOCK_SIZE) as f64) / BLOCK_SIZE as f64;
			y = (y - (y as uint % BLOCK_SIZE) as f64) / BLOCK_SIZE as f64;
			self.grid.insert(Block::new(Location::new(x as uint, y as uint)))
		}
		debug!("released mouse button: {}", args.button);
	}

	fn mouse_move(&mut self, args: &MouseMoveArgs) {
		self.mouse_loc = (args.x, args.y);
	}

	fn render(&mut self, args: &mut RenderArgs) {
		(&mut self.gl).viewport(0, 0, args.width as i32, args.height as i32);
		let ref c = Context::abs(args.width as f64, args.height as f64);
		c.rgb(1.0, 1.0, 1.0).clear(&mut self.gl);

		if self.started {
			if self.count > 0 {
				self.count -= 1;
			} else {
				self.count = 30;
				self.render_logic();
			}
		}

		self.grid.render(&mut self.gl, c);
	}
}

#[start]
fn start(argc: int, argv: **u8) -> int {
	native::start(argc, argv, main)
}

fn main() {
	assert!(WINDOW_WIDTH % BLOCK_SIZE == 0);
	assert!(WINDOW_HEIGHT % BLOCK_SIZE == 0);
	let mut window = GameWindowSDL2::new(
		GameWindowSettings {
			title: "Conway's Game of Life".to_string(),
			size: [WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32],
			fullscreen: false,
			exit_on_esc: true
		}
	);
	let mut app = App::new();
	let game_iter_settings = GameIteratorSettings {
		updates_per_second: 120,
		max_frames_per_second: 60
	};
	app.run(&mut window, &game_iter_settings);
}
