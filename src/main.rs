use console::Term;
use console::Key;

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
	let mut game = MSGame::new(16, 16);
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
	fn new(width: usize, height: usize) -> Self {
		let size = width * height;
		let mut board = Vec::<Tile>::with_capacity(size);
		board.resize_with(size, || {Tile::new()});
		
		Self {
			cursor_x: 0,
			cursor_y: 0,
			width,
			height,
			board,
			flags: 0,
			mines: 0,
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
						
						let mut tile = self.get(x, y);
						if let TileContents::Number(count) = tile.contents {
							tile.contents = TileContents::Number(count + 1);
							self.set(x, y, tile);
						}

					}
					self.mines += 1;
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
							//unexplored space that is safe
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
		let mut x = self.cursor_x as i32;
		let mut y = self.cursor_y as i32;

		match direction {
			Direction::Up	=> y -= 1,
			Direction::Down	=> y += 1,
			Direction::Left	=> x -= 1,
			Direction::Right=> x += 1,
		}

		x = (x + self.width as i32) % self.width as i32;
		y = (y + self.height as i32) % self.height as i32;
		self.cursor_x = x as usize;
		self.cursor_y = y as usize;
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
			return Tile::new_void();
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
	Void,//out of bounds
}

#[derive(Copy, Clone)]
enum TileVis {
	Hidden,
	Flag,
	Open,
}


impl Tile {
	fn new() -> Self {
		let contents = if rand::random::<f32>() > 0.85 {
			TileContents::Mine
		} else {
			TileContents::Number(0)
		};

		Self {
			contents,
			visibility: TileVis::Hidden
		}
	}
	fn new_void() -> Self {
		Self {
			contents: TileContents::Void,
			visibility: TileVis::Open,
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
					TileContents::Void => "MARKED AS OUT OF BOUNDS".into(),
				}
			},
			TileVis::Flag => "F".into(),
		}
	}
}