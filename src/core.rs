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
                return true;
            }
        }
        return false;
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
        }
    }

    /// merge player to board and set new player
    pub fn lock_player(&mut self) {
        self.player.get_cells().into_iter().for_each(|(x, y)| {
            self.board.grid[y as usize][x as usize] = Cell::Mino(self.player.mino_type);
        });
        self.clear_lines();
        self.player = Mino::new(self.bags.next(), &self.board);
        self.can_hold = true;
    }

    pub fn clear_lines(&mut self) {
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
            self.player = Mino::new(self.hold.unwrap_or(self.bags.next()), &self.board);
            self.hold = Some(prev_type);
            self.can_hold = false
        }
    }
}
