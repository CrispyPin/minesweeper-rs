use std::ops::Add;
use std::ops::Rem;

use console::Term;
use console::Key;
use rand::prelude::SliceRandom;
use rand::thread_rng;

const NEIGHBOR_OFFSETS:[(i32, i32); 8] = [(-1,-1),(0,-1),(1,-1),(-1,0),(1,0),(-1,1),(0,1),(1,1)];

enum Action {
	Continue,
	Lose,
	Win,
	Quit,
}

enum Direction {
	Up,
	Down,
	Left,
	Right
}

fn main() {
	let stdout = Term::buffered_stdout();
	let mut game = MSGame::new(16, 16, 32);
	game.init();
	game.draw(&stdout);

	loop {
		let action = game.process_key(stdout.read_key().expect("failed to read key"));
		game.draw(&stdout);
		match action {
			Action::Quit => break,
			Action::Lose => {
				println!("GAME OVER!");
				break;
			},
			Action::Win => {
				println!("YOU WIN!");
				break;
			},
			Action::Continue => (),
		}
	}
}

struct MSGame {
	width: usize,
	height: usize,
	cursor_x: usize,
	cursor_y: usize,
	board: Vec<Tile>,
	mines: u32,
	flags: u32,
}

impl MSGame {
	fn new(width: usize, height: usize, mines: u32) -> Self {
		let size = width * height;
		let mut board = Vec::<Tile>::with_capacity(size);
		let empty_tiles = (size as u32 - mines) as usize;
		board.resize_with(empty_tiles, || {Tile::new(false)});
		for _ in 0..mines {
			board.push(Tile::new(true));
		}
		board.shuffle(&mut thread_rng());

		
		Self {
			cursor_x: 0,
			cursor_y: 0,
			width,
			height,
			board,
			flags: 0,
			mines,
		}
	}

	fn init(&mut self) {
		for center_y in 0..self.height {
			for center_x in 0..self.width {
				let tile = self.get(center_x, center_y);
				if let TileContents::Mine = tile.contents {
					for (dx, dy) in NEIGHBOR_OFFSETS {
						let (x, y) = (dx + center_x as i32, dy + center_y as i32);
						let x = x as usize;
						let y = y as usize;
						
						if !self.valid_pos(x, y) {
							continue;
						}
						let mut tile = self.get(x, y);
						if let TileContents::Number(count) = tile.contents {
							tile.contents = TileContents::Number(count + 1);
							self.set(x, y, tile);
						}

					}
				}
			}
		}
	}

	fn process_key(&mut self, key: Key) -> Action{
		match key {
			Key::ArrowUp    => self.move_cursor(Direction::Up),
			Key::ArrowLeft  => self.move_cursor(Direction::Left),
			Key::ArrowDown  => self.move_cursor(Direction::Down),
			Key::ArrowRight => self.move_cursor(Direction::Right),
			Key::Char('f') => self.flag(),
			Key::Char(' ') => self.open(),
			Key::Char('q') => return Action::Quit,
			_ => (),
		}
		self.state()
	}

	fn state(&self) -> Action {
		let mut explored = true;
		for y in 0..self.height {
			for x in 0..self.width {
				let tile = self.get(x, y);
				match tile.visibility {
					TileVis::Open => {
						if let TileContents::Mine = tile.contents {
							return Action::Lose;
						}
					},
					TileVis::Hidden => {
						if let TileContents::Number(_) = tile.contents {
							explored = false;
						}
					},
					_ => (),
				}
			}
		}
		if explored {
			Action::Win
		}
		else {
			Action::Continue
		}
	}

	fn open_single(&mut self, x: usize, y: usize) {
		let i = self.index_of(x, y);
		let tile = &mut self.board[i];
		if let TileVis::Hidden = tile.visibility {
			tile.visibility = TileVis::Open;
		}
	}

	fn open(&mut self) {
		let mut queue = vec![(self.cursor_x, self.cursor_y)];
		let mut i = 0;
		loop {
			if i >= queue.len() {
				break;
			}
			let (x, y) = queue[i];
			let tile = self.get(x, y);
			
			if let TileVis::Hidden = tile.visibility {
				self.open_single(x, y);
				if let TileContents::Number(0) = tile.contents {
					for offset in NEIGHBOR_OFFSETS {
						let tx = (x as i32 + offset.0) as usize; // negatives end up giant but that is fine for bound checking
						let ty = (y as i32 + offset.1) as usize;
						if !self.valid_pos(tx, ty) {
							continue;
						}
						let target = self.get(tx, ty);
						if let TileVis::Open = target.visibility {
							continue;
						}
						queue.push((tx, ty));
					}
				}
			}
			i += 1;
		}
		
	}

	fn flag(&mut self) {
		let i = self.index_of(self.cursor_x, self.cursor_y);
		let tile = &mut self.board[i];

		match tile.visibility {
			TileVis::Flag => {
				tile.visibility = TileVis::Hidden;
				self.flags -= 1;
			},
			TileVis::Hidden => {
				tile.visibility = TileVis::Flag;
				self.flags += 1;
			},
			TileVis::Open => (),
		}
	}

	fn move_cursor(&mut self, direction: Direction) {
		match direction {
			Direction::Up	=> self.cursor_y = self.cursor_y
				.wrapping_sub(1)
				.min(self.height - 1),
			Direction::Down	=> self.cursor_y = self.cursor_y
				.add(1)
				.rem(self.height),
			Direction::Left	=> self.cursor_x = self.cursor_x
				.wrapping_sub(1)
				.min(self.width - 1),
			Direction::Right=> self.cursor_x = self.cursor_x
				.add(1)
				.rem(self.width),
		}
	}

	fn draw(&self, stdout: &Term) {
		stdout.clear_screen().expect("failed to clear");
		stdout.flush().expect("failed to flush");
		for y in 0..self.height {
			if self.cursor_y == y {
				cell_gap(self.cursor_x, -1);
			} else {
				print!(" ");
			}
			for x in 0..self.width {
				let tile = self.get(x, y);

				if self.cursor_y == y {
					print!("{}", tile.draw());
					cell_gap(self.cursor_x, x as i32);
				}
				else {
					print!("{} ", tile.draw());
				}
			}
			println!();
		}
		println!();
		println!("Mines: {}, Flags: {}, Remaining: {}", self.mines, self.flags, self.mines - self.flags);

		fn cell_gap(cursor_x: usize, x: i32) {
			let cx = cursor_x as i32;
			if cx == x {
				print!(")");
			}
			else if cx == x + 1 {
				print!("(");
			}
			else {
				print!(" ");
			}
		}
	}

	fn get(&self, x: usize, y: usize) -> Tile {
		if !self.valid_pos(x, y) {
			panic!("invalid get pos");
		}
		let i = self.index_of(x, y);
		self.board[i]
	}
	
	fn set(&mut self, x: usize, y: usize, tile: Tile) {
		if !self.valid_pos(x, y) {
			return;
		}
		let i = self.index_of(x, y);
		self.board[i] = tile;
	}

	fn valid_pos(&self, x: usize, y: usize) -> bool{
		x < self.width && y < self.height
	}

	fn index_of(&self, x: usize, y: usize) -> usize{
		x + y * self.width
	}
}

#[derive(Copy, Clone)]
struct Tile {
	contents: TileContents,
	visibility: TileVis
}

#[derive(Copy, Clone)]
enum TileContents {
	Number(u8),
	Mine,
}

#[derive(Copy, Clone)]
enum TileVis {
	Hidden,
	Flag,
	Open,
}


impl Tile {
	fn new(mine: bool) -> Self {
		let contents = if mine {
			TileContents::Mine
		} else {
			TileContents::Number(0)
		};
		Self {
			contents,
			visibility: TileVis::Hidden
		}
	}

	fn draw(&self) -> String {
		match &self.visibility {
			TileVis::Hidden => "#".into(),
			TileVis::Open => {
				match self.contents {
					TileContents::Mine => "*".into(),
					TileContents::Number(0) => " ".into(),
					TileContents::Number(num) => format!("{}", num),
				}
			},
			TileVis::Flag => "F".into(),
		}
	}
}
