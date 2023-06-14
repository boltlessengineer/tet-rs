use std::io;

use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::core::{Bag, Cell, Game, MinoType, GRID_HEIGHT, GRID_WIDTH};

const CELL_WIDTH: u16 = 3;

// trait GetSpans {
//     fn get_spans(&self) -> Vec<Span>;
// }

impl Cell {
    fn get_spans(&self) -> Vec<Span<'static>> {
        // let cube_text = "+";
        let cube_text = "   ";
        let empty_text = "   ";
        let s = Style::default();
        match self {
            Cell::Mino(minotype) => {
                let style = match minotype {
                    MinoType::I => s.bg(Color::LightBlue),
                    MinoType::J => s.bg(Color::Blue),
                    MinoType::L => s.bg(Color::LightYellow),
                    MinoType::O => s.bg(Color::Yellow),
                    MinoType::S => s.bg(Color::LightGreen),
                    MinoType::T => s.bg(Color::Magenta),
                    MinoType::Z => s.bg(Color::Red),
                };
                vec![Span::styled(cube_text, style)]
            }
            Cell::Ghost => vec![Span::styled(cube_text, s.bg(Color::Black))],
            Cell::_Garbage => vec![Span::styled(cube_text, s.bg(Color::Gray))],
            Cell::Empty => vec![Span::raw(empty_text)],
        }
    }
}

trait ToSpans {
    fn get_spans(&self) -> Vec<Spans<'static>>;
}

impl ToSpans for Option<MinoType> {
    fn get_spans(&self) -> Vec<Spans<'static>> {
        let mut grid = [[Cell::Empty; 4]; 2];
        if let Some(mino_type) = self {
            mino_type.get_cells().into_iter().for_each(|(x, y)| {
                grid[*y as usize][(*x + 1) as usize] = Cell::Mino(*mino_type);
            });
        }
        let mut grid_text = Vec::new();
        for line in grid.into_iter().rev() {
            let mut line_spans = Vec::new();
            for cell in line {
                let cell_spans = cell.get_spans();
                line_spans.extend(cell_spans.into_iter());
            }
            grid_text.push(Spans::from(line_spans))
        }
        return grid_text;
    }
}

impl ToSpans for Bag {
    fn get_spans(&self) -> Vec<Spans<'static>> {
        let mut grid_text = Vec::new();
        self.list.iter().rev().for_each(|mino_type| {
            grid_text.extend(Some(*mino_type).get_spans().into_iter());
            grid_text.extend((vec![Spans::from(Cell::Empty.get_spans())]).into_iter());
        });
        return grid_text;
    }
}

pub struct UI {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl UI {
    pub fn new() -> crossterm::Result<UI> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout
            .execute(EnterAlternateScreen)?
            .execute(EnableMouseCapture)?
            .execute(PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
            ))?;
        let backend = CrosstermBackend::new(stdout);
        return Ok(UI {
            terminal: Terminal::new(backend)?,
        });
    }

    pub fn render(&mut self, game: &Game) -> crossterm::Result<()> {
        // create board widget
        let mut grid = game.board.grid.clone();
        game.player
            .get_ghost()
            .get_cells()
            .into_iter()
            .for_each(|(x, y)| {
                grid[y as usize][x as usize] = Cell::Ghost;
            });
        game.player.get_cells().into_iter().for_each(|(x, y)| {
            grid[y as usize][x as usize] = Cell::Mino(game.player.mino_type);
        });
        let mut grid_text = Vec::new();
        for line in grid.into_iter().rev() {
            let mut line_spans = Vec::new();
            for cell in line {
                let cell_spans = cell.get_spans();
                line_spans.extend(cell_spans.into_iter());
            }
            grid_text.push(Spans::from(line_spans))
        }
        let board_widget = Paragraph::new(grid_text)
            .block(Block::default().title("TETRIS").borders(Borders::all()));
        let board_area = Rect::new(0, 0, GRID_WIDTH * CELL_WIDTH + 2, GRID_HEIGHT + 2);

        let hover_widget = Paragraph::new(game.hold.get_spans())
            .block(Block::default().title("Hold").borders(Borders::all()));
        let hover_area = Rect::new(GRID_WIDTH * CELL_WIDTH + 2, 0, 4 * CELL_WIDTH + 2, 2 + 2);

        let bag_widget = Paragraph::new(game.bags.get_spans())
            .block(Block::default().title("Next").borders(Borders::all()));
        let bag_area = Rect::new(
            GRID_WIDTH * CELL_WIDTH + 2,
            4,
            4 * CELL_WIDTH + 2,
            3 * 6 + 2,
        );

        self.terminal.draw(|f| {
            f.render_widget(board_widget, board_area);
            f.render_widget(hover_widget, hover_area);
            f.render_widget(bag_widget, bag_area);
        })?;
        return Ok(());
    }

    pub fn exit(&mut self) -> crossterm::Result<()> {
        disable_raw_mode()?;
        self.terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)?
            .execute(DisableMouseCapture)?
            .execute(PopKeyboardEnhancementFlags)?;
        return Ok(());
    }
}
