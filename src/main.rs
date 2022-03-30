use std::ops::Add;
use std::ops::Rem;

use console::Term;
use console::Key;

use rand::prelude::SliceRandom;
use rand::thread_rng;

const NEIGHBOR_OFFSETS: [(i32, i32); 8] = [(-1,-1),(0,-1),(1,-1),(-1,0),(1,0),(-1,1),(0,1),(1,1)];

enum TurnResult {
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
			TurnResult::Quit => break,
			TurnResult::Lose => {
				println!("GAME OVER!");
				break;
			},
			TurnResult::Win => {
				println!("YOU WIN!");
				break;
			},
			TurnResult::Continue => (),
		}
	}
}

struct MSGame {
	width: usize,
	height: usize,
	cursor_x: usize,
	cursor_y: usize,
	board: Vec<Tile>,
	mines: usize,
	flags: usize,
}

impl MSGame {
	fn new(width: usize, height: usize, mines: usize) -> Self {
		let size = width * height;
		let mut board = Vec::<Tile>::with_capacity(size);
		
		let empty_tiles = size.saturating_sub(mines);

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
						let x = center_x.wrapping_add(dx as usize);
						let y = center_y.wrapping_add(dy as usize);
						
						if !self.valid_pos(x, y) {
							continue;
						}
						
						let tile = self.get_mut(x, y);
						if let TileContents::Number(count) = tile.contents {
							tile.contents = TileContents::Number(count + 1);
						}

					}
				}
			}
		}
	}

	fn process_key(&mut self, key: Key) -> TurnResult{
		match key {
			Key::ArrowUp    => self.move_cursor(Direction::Up),
			Key::ArrowLeft  => self.move_cursor(Direction::Left),
			Key::ArrowDown  => self.move_cursor(Direction::Down),
			Key::ArrowRight => self.move_cursor(Direction::Right),
			Key::Char('f') => self.flag(),
			Key::Char(' ') => self.open(),
			Key::Char('q') => return TurnResult::Quit,
			_ => (),
		}
		self.state()
	}

	fn state(&self) -> TurnResult {
		let mut explored = true;
		for y in 0..self.height {
			for x in 0..self.width {
				let tile = self.get(x, y);
				match tile.visibility {
					TileVis::Open => {
						if let TileContents::Mine = tile.contents {
							return TurnResult::Lose;
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
			TurnResult::Win
		}
		else {
			TurnResult::Continue
		}
	}

	fn open_single(&mut self, x: usize, y: usize) {
		let i = self.index_of(x, y);
		let tile = &mut self.board[i];
		if let TileVis::Hidden = tile.visibility {
			tile.visibility = TileVis::Open;
		}
	}

	// flood fill to open all adjacent clear tiles
	fn open(&mut self) {
		let mut queue = vec![(self.cursor_x, self.cursor_y)];
		let mut i = 0;
		
		while i < queue.len() {
			let (x, y) = queue[i];
			let tile = self.get(x, y);
			
			if let TileVis::Hidden = tile.visibility {
				self.open_single(x, y);
				// if this tile is a 0, add its neighbors to the queue (if they are not already open)
				if let TileContents::Number(0) = tile.contents {
					for (dx, dy) in NEIGHBOR_OFFSETS {
						let target_x = x.wrapping_add(dx as usize);
						let target_y = y.wrapping_add(dy as usize);
						if !self.valid_pos(target_x, target_y) {
							continue;
						}
						let target = self.get(target_x, target_y);
						if let TileVis::Open = target.visibility {
							continue;
						}
						queue.push((target_x, target_y));
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
		stdout.clear_screen().unwrap();
		stdout.flush().unwrap();

		for row in 0..self.height {
			cell_gap(self.cursor_x, self.cursor_y, usize::MAX, row);
			
			for col in 0..self.width {
				let tile = self.get(col, row);
				
				print!("{}", tile.draw());
				cell_gap(self.cursor_x, self.cursor_y, col, row);
			}
			println!();
		}
		println!();
		println!("Mines: {}, Flags: {}, Remaining: {}", self.mines, self.flags, self.mines - self.flags);

		fn cell_gap(cursor_x: usize, cursor_y: usize, col: usize, row: usize) {
			if cursor_y != row {
				print!(" ");
				return;
			}
			match cursor_x.wrapping_sub(col) {
				1 => print!("("),
				0 => print!(")"),
				_ => print!(" "),
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

	fn get_mut(&mut self, x: usize, y: usize) -> &mut Tile {
		if !self.valid_pos(x, y) {
			panic!("invalid get pos");
		}
		let i = self.index_of(x, y);
		&mut self.board[i]
	}
	
	fn valid_pos(&self, x: usize, y: usize) -> bool {
		x < self.width && y < self.height
	}

	fn index_of(&self, x: usize, y: usize) -> usize {
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
