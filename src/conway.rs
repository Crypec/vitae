use coffee::graphics::{Color, Frame, Mesh, Point, Rectangle, Shape, Window};
use coffee::input::keyboard::KeyCode;
use coffee::input::{self, keyboard, mouse, ButtonState, Input};
use coffee::load::Task;
use coffee::{Game, Timer};

pub const WINDOW_SIZE_X: usize = 1024;
pub const WINDOW_SIZE_Y: usize = WINDOW_SIZE_X;

pub const CELL_SIZE: usize = 10;

const CELL_COUNT_X: usize = WINDOW_SIZE_X / CELL_SIZE;
const CELL_COUNT_Y: usize = WINDOW_SIZE_Y / CELL_SIZE;

const GRID_COLOR: Color = Color::BLACK;

type Board = Vec<Vec<Cell>>;

pub const KERNEL: [(isize, isize); 8] = [
	(-1, -1),
	(0, -1),
	(1, -1),
	(-1, 0),
	(1, 0),
	(-1, 1),
	(0, 1),
	(1, 1),
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Cell {
	Dead,
	Alive,
}

#[derive(Debug)]
pub struct CustomInput {
	mode: Mode,
	mouse_points: Vec<Point>,
	action: InputAction,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputAction {
	PlaceAlive,
	PlaceDead,
	Pause,
	None,
}

impl Input for CustomInput {
	fn new() -> Self {
		Self {
			mouse_points: vec![],
			action: InputAction::None,
			mode: Mode::Editor(EditorMode::Drawing),
		}
	}

	fn update(&mut self, event: input::Event) {
		match event {
			input::Event::Mouse(mouse_event) => match mouse_event {
				mouse::Event::CursorMoved { x, y } => {
					if self.mode == Mode::Simulation {
						return;
					}
					self.mouse_points.push(Point::new(x, y));
				}
				mouse::Event::Input {
					state: input::ButtonState::Pressed,
					button,
				} => {
					match button {
						mouse::Button::Left => {
							self.action = match self.action {
								InputAction::PlaceAlive => InputAction::None,
								_ => InputAction::PlaceAlive,
							}
						}
						mouse::Button::Right => {
							self.action = match self.action {
								InputAction::PlaceDead => InputAction::None,
								_ => InputAction::PlaceDead,
							}
						}
						_ => {},
					}
				},
				_ => {},
			},
			input::Event::Keyboard(keyboard::Event::Input {
				key_code: KeyCode::P,
				state: ButtonState::Pressed,
			}) => self.action = InputAction::Pause,
			_ => {}
		}
	}

	fn clear(&mut self) {}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Mode {
	Simulation,
	Editor(EditorMode),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EditorMode {
	Drawing,
	Moving,
}

#[derive(Debug)]
pub struct Conway {
	current_board: Board,
	new_board: Board,
	mode: Mode,
}

impl Conway {
	pub fn new() -> Self {
		Self {
			current_board: vec![vec![Cell::Dead; CELL_COUNT_X]; CELL_COUNT_Y],
			new_board: vec![vec![Cell::Dead; CELL_COUNT_X]; CELL_COUNT_Y],
			mode: Mode::Editor(EditorMode::Drawing),
		}
	}

	pub fn count_neighbors(&self, x: usize, y: usize) -> usize {
		let mut n = 0;
		for (dx, dy) in &KERNEL {
			let x = (x as isize + dx) as usize;
			let y = (y as isize + dy) as usize;

			if self.out_of_bounds(x, y) {
				continue;
			}
			if let Cell::Alive = self.current_board[y][x] {
				n += 1;
			}
		}
		n
	}

	pub fn update_board_state(&mut self) {
		for (y, row) in self.current_board.iter().enumerate() {
			for (x, cell) in row.iter().enumerate() {
				let n = self.count_neighbors(x, y);
				let new_cell = match cell {
					Cell::Alive if n < 2 => Cell::Dead,
					Cell::Alive if n == 2 || n == 3 => Cell::Alive,
					Cell::Alive if n > 3 => Cell::Dead,
					Cell::Dead if n == 3 => Cell::Alive,
					_ => Cell::Dead,
				};
				self.new_board[y][x] = new_cell;
			}
		}
		std::mem::swap(&mut self.current_board, &mut self.new_board);
		self.clear_new_board();
	}

	// NOTE(Simon): we rely on overflowing the usize for checking indices which are out of bound in the negative direction
	const fn out_of_bounds(&self, x: usize, y: usize) -> bool {
		x >= CELL_COUNT_X || y >= CELL_COUNT_Y
	}

	fn clear_new_board(&mut self) {
		for row in &mut self.new_board {
			unsafe {
				let row_ptr = row.as_mut_ptr();
				std::ptr::write_bytes(row_ptr, 0, CELL_COUNT_X);
			}
		}
	}

	fn draw_cells(&mut self, mesh: &mut Mesh) {
		for (y, row) in self.current_board.iter_mut().enumerate() {
			for (x, cell) in row.iter_mut().enumerate() {
				if let Cell::Alive = cell {
					mesh.fill(
						Shape::Rectangle(Rectangle {
							x: (x * CELL_SIZE) as f32,
							y: (y * CELL_SIZE) as f32,
							width: CELL_SIZE as f32,
							height: CELL_SIZE as f32,
						}),
						Color::BLACK,
					);
				}
			}
		}
	}

	pub fn draw_grid(mesh: &mut Mesh) {
		let x_bound = WINDOW_SIZE_X / CELL_SIZE as usize;
		let y_bound = WINDOW_SIZE_Y / CELL_SIZE as usize;
		for i in 0..x_bound {
			let i = i as f32;
			let line = Shape::Polyline {
				points: vec![
					Point::new(i * CELL_SIZE as f32, 0.0),
					Point::new(i * CELL_SIZE as f32, WINDOW_SIZE_Y as f32),
				],
			};
			mesh.stroke(line, GRID_COLOR, 1.0);
		}
		for i in 0..y_bound {
			let i = i as f32;
			let line = Shape::Polyline {
				points: vec![
					Point::new(0.0, i * CELL_SIZE as f32),
					Point::new(WINDOW_SIZE_X as f32, i * CELL_SIZE as f32),
				],
			};
			mesh.stroke(line, GRID_COLOR, 1.0);
		}
	}

	pub fn toggle_mode(&mut self) {
		self.mode = match self.mode {
			Mode::Editor(_) => Mode::Simulation,
			Mode::Simulation => Mode::Editor(EditorMode::Drawing),
		};
	}
}

impl Game for Conway {
	const TICKS_PER_SECOND: u16 = 10;
	type Input = CustomInput;
	type LoadingScreen = ();

	fn load(_window: &Window) -> Task<Self> {
		Task::succeed(Self::new)
	}

	fn update(&mut self, _: &Window) {
		if let Mode::Editor(_) = self.mode {
			return;
		}
		self.update_board_state();
	}

	fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
		frame.clear(Color::WHITE);
		let mut mesh = Mesh::new();
		self.draw_cells(&mut mesh);
		Self::draw_grid(&mut mesh);
		mesh.draw(&mut frame.as_target());
	}

	fn interact(&mut self, input: &mut Self::Input, _: &mut Window) {
		if input.action == InputAction::Pause {
			input.action = InputAction::None;
			self.toggle_mode();
			input.mode = self.mode;
		}
		if self.mode == Mode::Simulation {
			return;
		}
		while let Some(p) = input.mouse_points.pop() {
			let x = (p.x / CELL_SIZE as f32) as usize;
			let y = (p.y / CELL_SIZE as f32) as usize;
			match input.action {
				InputAction::PlaceAlive => self.current_board[y][x] = Cell::Alive,
				InputAction::PlaceDead => self.current_board[y][x] = Cell::Dead,
				InputAction::Pause => self.toggle_mode(),
				InputAction::None => {},
			}
		}
	}
}
