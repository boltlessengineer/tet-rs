use std::time::Instant;

use rand::seq::SliceRandom;

pub const GRID_WIDTH: u16 = 10;
pub const GRID_HEIGHT: u16 = 24;

type Pos = (i8, i8);
const TETRIMINO: usize = 4;

const I_CELLS: [Pos; TETRIMINO] = [(-1, 0), (0, 0), (1, 0), (2, 0)];
const J_CELLS: [Pos; TETRIMINO] = [(-1, 1), (-1, 0), (0, 0), (1, 0)];
const L_CELLS: [Pos; TETRIMINO] = [(-1, 0), (0, 0), (1, 0), (1, 1)];
const O_CELLS: [Pos; TETRIMINO] = [(0, 1), (0, 0), (1, 0), (1, 1)];
const S_CELLS: [Pos; TETRIMINO] = [(-1, 0), (0, 0), (0, 1), (1, 1)];
const T_CELLS: [Pos; TETRIMINO] = [(-1, 0), (0, 0), (0, 1), (1, 0)];
const Z_CELLS: [Pos; TETRIMINO] = [(-1, 1), (0, 1), (0, 0), (1, 0)];

static JLSTZ_OFFSETS: [[Pos; 5]; 4] = [
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
];
static I_OFFSETS: [[Pos; 5]; 4] = [
    [(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
    [(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
    [(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
    [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
];
static O_OFFSETS: [[Pos; 5]; 4] = [
    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)], // No further offset data required
    [(0, -1), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(-1, -1), (0, 0), (0, 0), (0, 0), (0, 0)],
    [(-1, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
];
static Gravity: [f64; 20] = [
    1.00000, // 1
    0.79300, // 2
    0.61780, // 3
    0.47273, // 4
    0.35520, // 5
    0.26200, // 6
    0.18968, // 7
    0.13473, // 8
    0.09388, // 9
    0.06415, // 10
    0.04298, // 11
    0.02822, // 12
    0.01815, // 13
    0.01144, // 14
    0.00706, // 15
    0.00426, // 16
    0.00252, // 17
    0.00146, // 18
    0.00082, // 19
    0.00046, // 20
];

// TODO: rename to Up, Right, Down, Left
// also use on das_shift(&self, dir: Direction, game: &Game)
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    /// Zero : Initial state
    Z,
    /// Right : Single clockwise rotation
    R,
    /// Double : 180' rotation
    D,
    /// Left : Single counter clockwise rotation
    L,
}
impl std::ops::Add<Direction> for Direction {
    type Output = Direction;
    fn add(self, rhs: Direction) -> Self::Output {
        use Direction::*;
        let n = self as usize + rhs as usize;
        match n % 4 {
            0 => Z,
            1 => R,
            2 => D,
            _ => L,
        }
    }
}
impl std::ops::AddAssign<Direction> for Direction {
    fn add_assign(&mut self, rhs: Direction) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MinoType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl MinoType {
    pub fn get_cells(&self) -> &[Pos; TETRIMINO] {
        match self {
            MinoType::I => &I_CELLS,
            MinoType::J => &J_CELLS,
            MinoType::L => &L_CELLS,
            MinoType::O => &O_CELLS,
            MinoType::S => &S_CELLS,
            MinoType::T => &T_CELLS,
            MinoType::Z => &Z_CELLS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mino {
    pub mino_type: MinoType,
    pub direction: Direction,
    pub ghost_y: i8,
    pub x: i8,
    pub y: i8,
}

impl Mino {
    pub fn new(mino_type: MinoType, board: &Board) -> Mino {
        let mut mino = Mino {
            mino_type,
            direction: Direction::Z,
            ghost_y: 18,
            x: 4,
            y: 18,
        };
        mino.update_ghost_y(board);
        return mino;
    }

    pub fn get_cells(&self) -> [Pos; TETRIMINO] {
        self.mino_type
            .get_cells()
            .map(|(x, y)| match self.direction {
                Direction::Z => (self.x + x, self.y + y),
                Direction::R => (self.x + y, self.y - x),
                Direction::D => (self.x - x, self.y - y),
                Direction::L => (self.x - y, self.y + x),
            })
    }

    pub fn collides(&self, board: &Board) -> bool {
        self.get_cells()
            .into_iter()
            .any(|(x, y)| !board.is_empty(x, y))
    }

    /// returns true if player moved
    pub fn shift(&mut self, x: i8, y: i8, board: &Board) -> bool {
        let mut temp = self.clone();
        temp.x += x;
        temp.y += y;
        if !temp.collides(board) {
            *self = temp;
            self.update_ghost_y(board);
            return true;
        }
        return false;
    }

    // FIX: das should not work like this!!!
    // think if ARR != 0
    pub fn das_shift(&mut self, direction: Direction, board: &Board) -> bool {
        match direction {
            // HACK: something better than unreachable
            Direction::Z => unreachable!(),
            Direction::R => {
                let prev_x = self.x;
                self.x += 1;
                while !self.collides(board) {
                    self.x += 1;
                }
                self.x -= 1;
                return prev_x != self.x;
            }
            Direction::D => unreachable!(),
            Direction::L => {
                let prev_x = self.x;
                self.x -= 1;
                while !self.collides(board) {
                    self.x -= 1;
                }
                self.x += 1;
                return prev_x != self.x;
            }
        }
    }

    pub fn update_ghost_y(&mut self, board: &Board) {
        let mut ghost = self.clone();
        ghost.y -= 1;
        while !ghost.collides(board) {
            ghost.y -= 1;
        }
        ghost.y += 1;
        self.ghost_y = ghost.y;
    }

    pub fn get_ghost(&self) -> Mino {
        let mut ghost = self.clone();
        ghost.y = self.ghost_y;
        return ghost;
    }

    pub fn rotate(&mut self, direction: Direction, board: &Board) -> bool {
        let mut temp = self.clone();
        temp.direction += direction;
        let offset_table = match self.mino_type {
            MinoType::I => I_OFFSETS,
            MinoType::O => O_OFFSETS,
            _ => JLSTZ_OFFSETS,
        };
        let pre_offset = offset_table[self.direction as usize];
        let post_offset = offset_table[temp.direction as usize];
        for i in 0..5 {
            let mut t = temp.clone();
            t.x += pre_offset[i].0 - post_offset[i].0;
            t.y += pre_offset[i].1 - post_offset[i].1;
            if !t.collides(board) {
                *self = t;
                self.update_ghost_y(board);
                // retrun false when mino is O
                let is_o_mino = matches!(self.mino_type, MinoType::O);
                return !is_o_mino;
            }
        }
        return false;
    }

    pub fn is_bottom(&self) -> bool {
        return self.y == self.ghost_y;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Cell {
    Mino(MinoType),
    Ghost,
    _Garbage,
    Empty,
}

impl Cell {
    pub fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Board {
    pub grid: [[Cell; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
}

impl Board {
    pub fn new() -> Board {
        Board {
            grid: [[Cell::Empty; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
        }
    }

    pub fn is_in_bounds(&self, x: i8, y: i8) -> bool {
        return x < GRID_WIDTH as i8 && y < GRID_HEIGHT as i8 && x >= 0 && y >= 0;
    }

    pub fn is_empty(&self, x: i8, y: i8) -> bool {
        if self.is_in_bounds(x, y) {
            self.grid[y as usize][x as usize].is_empty()
        } else {
            false
        }
    }
}

pub struct Bag {
    pub list: Vec<MinoType>,
}

fn get_bag() -> [MinoType; 7] {
    use MinoType::*;
    let mut arr = [I, J, L, O, S, T, Z];
    arr.shuffle(&mut rand::thread_rng());
    return arr;
}

impl Bag {
    pub fn new() -> Bag {
        return Bag {
            list: Vec::from(get_bag()),
        };
    }
    pub fn next(&mut self) -> MinoType {
        if let Some(mino) = self.list.pop() {
            return mino;
        } else {
            self.list.extend_from_slice(&get_bag());
            return self.list.pop().unwrap();
        }
    }
}

pub struct Game {
    // TODO: change these to pointers (and use `iter()` instead of `into_iter()`)
    pub board: Board,
    pub player: Mino,
    pub hold: Option<MinoType>,
    pub bags: Bag,
    can_hold: bool,
    pub last_touch: Option<Instant>,
    /// last instant when user moved Mino
    pub last_move: Option<Instant>,
    pub canceled_drop: u8,
    /// Some() when das is charging
    /// value is initial time when DAS charging is started
    pub das_charge_left: Option<Instant>,
    /// Some() when das is charging
    /// value is initial time when DAS charging is started
    pub das_charge_right: Option<Instant>,
}

impl Game {
    pub fn new() -> Game {
        let mut bag = Bag::new();
        let board = Board::new();
        let player = Mino::new(bag.next(), &board);
        Game {
            board: Board::new(),
            player,
            hold: None,
            bags: bag,
            can_hold: true,
            last_touch: None,
            last_move: None,
            canceled_drop: 0,
            das_charge_left: None,
            das_charge_right: None,
        }
    }

    /// merge player to board and set new player
    /// this will hard-drop Mino if Mino isn't at bottom
    pub fn lock_player(&mut self) {
        self.player
            .shift(0, self.player.ghost_y - self.player.y, &self.board);
        self.player.get_cells().into_iter().for_each(|(x, y)| {
            self.board.grid[y as usize][x as usize] = Cell::Mino(self.player.mino_type);
        });
        self.clear_lines();
        self.player = Mino::new(self.bags.next(), &self.board);
        self.can_hold = true;
        self.last_touch = None;
        self.canceled_drop = 0;
    }

    fn clear_lines(&mut self) {
        let rows_to_clear: Vec<usize> = (0..GRID_HEIGHT as usize)
            .filter(|&row| self.board.grid[row].iter().all(|cell| !cell.is_empty()))
            .collect();
        let empty_line = [Cell::Empty; GRID_WIDTH as usize];
        rows_to_clear.into_iter().rev().for_each(|row| {
            for row in row..GRID_HEIGHT as usize - 1 {
                self.board.grid[row] = self.board.grid[row + 1];
            }
            *self.board.grid.last_mut().unwrap() = empty_line.clone();
        });
    }

    pub fn swap_hold(&mut self) {
        if self.can_hold {
            let prev_type = self.player.mino_type;
            if let Some(hold) = self.hold {
                self.player = Mino::new(hold, &self.board);
            } else {
                self.player = Mino::new(self.bags.next(), &self.board);
            }
            self.hold = Some(prev_type);
            self.can_hold = false
        }
    }

    // HACK: wait... two similar same name function for two separate structs?
    pub fn shift(&mut self, x: i8, y: i8) {
        let last_line = self.player.y;
        let success = self.player.shift(x, y, &self.board);
        let moved_down = last_line > self.player.y;
        self.move_reset(success, moved_down);
    }

    // FIX: auto lock isn't working for 15+ movements
    pub fn rotate(&mut self, direction: Direction) {
        let last_line = self.player.y;
        let success = self.player.rotate(direction, &self.board);
        let moved_down = last_line > self.player.y;
        self.move_reset(success, moved_down);
    }

    fn move_reset(&mut self, success: bool, move_down: bool) {
        if self.last_touch.is_none() {
            if self.player.is_bottom() {
                self.last_touch = Some(Instant::now());
            }
        } else {
            // Mino _was_ at bottom
            if success {
                // movment occured
                self.canceled_drop += 1;
                if self.player.is_bottom() {
                    // still at bottom, reset timer
                    self.last_touch = Some(Instant::now());
                } else {
                    // but not at bottom now, remove timer
                    self.last_touch = None;
                }
            }
        }
        if move_down {
            self.canceled_drop = 0;
        }
    }
}
