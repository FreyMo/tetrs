use std::{
    cmp::{max, min},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::{stdout, Stdout},
};

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
    Terminal,
};

use crate::game::{
    level::Level,
    state::{Field, GameState, Phase, Square, FIELD_HEIGHT, FIELD_WIDTH},
    tetromino::Tetromino,
};

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    previous_size: Rect,
    previous_hash: u64,
}

impl Default for Ui {
    fn default() -> Self {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).unwrap();

        Self {
            terminal,
            previous_size: Rect::default(),
            previous_hash: u64::MAX,
        }
    }
}

const HEIGHT: u16 = FIELD_HEIGHT as u16 + 2;
const LEVEL_WIDTH: u16 = 17;
const GAME_WIDTH: u16 = FIELD_WIDTH as u16 * 2 + 2;

fn level_area(offset: &Rect) -> Rect {
    Rect::new(offset.x, offset.y, LEVEL_WIDTH, HEIGHT)
}

fn game_area(offset: &Rect) -> Rect {
    Rect::new(offset.x + LEVEL_WIDTH, offset.y, GAME_WIDTH, HEIGHT)
}

fn right_area(offset: &Rect) -> Rect {
    Rect::new(
        offset.x + LEVEL_WIDTH + GAME_WIDTH,
        offset.y,
        GAME_WIDTH,
        HEIGHT,
    )
}

impl Ui {
    pub fn draw(&mut self, phase: &Phase) {
        if self.should_render(phase) {
            let frame = self
                .terminal
                .draw(|frame| draw_frame(phase, frame))
                .unwrap();
            let mut hasher = DefaultHasher::new();
            phase.hash(&mut hasher);

            self.previous_size = frame.area;
            self.previous_hash = hasher.finish();
        }
    }

    fn should_render(&self, phase: &Phase) -> bool {
        let size_changed = self.terminal.size().unwrap() != self.previous_size;

        // TODO: this will not work when changing from running to finished, as the hash does not change
        match phase {
            Phase::Menu(_) => size_changed,
            _ => {
                let mut hasher = DefaultHasher::new();
                phase.hash(&mut hasher);
                let hash = hasher.finish();

                self.previous_hash != hash || size_changed
            }
        }
    }
}

fn draw_frame(phase: &Phase, frame: &mut tui::Frame<CrosstermBackend<Stdout>>) {
    match phase {
        Phase::Menu(_) => draw_menu(frame),
        Phase::Running(running) => draw_tetrs(&running.state, frame),
        Phase::Finished(finished) => draw_tetrs(&finished.state, frame),
    };
}

fn get_centered_rect(size: &Rect) -> Rect {
    let width = min(size.width, LEVEL_WIDTH + GAME_WIDTH * 2);
    let height = min(size.height, HEIGHT);

    let x = ((size.width - width) / 2).saturating_sub(1);
    let y = ((size.height - height) / 2).saturating_sub(1);

    Rect {
        x,
        y,
        width,
        height,
    }
}

fn fixed_intersection(left: &Rect, other: &Rect) -> Rect {
    let x1 = max(left.x, other.x);
    let y1 = max(left.y, other.y);
    let x2 = min(left.x + left.width, other.x + other.width);
    let y2 = min(left.y + left.height, other.y + other.height);

    Rect {
        x: x1,
        y: y1,
        width: x2.saturating_sub(x1),
        height: y2.saturating_sub(y1),
    }
}

fn draw_tetrs(state: &GameState, frame: &mut tui::Frame<CrosstermBackend<Stdout>>) {
    let rect = get_centered_rect(&frame.size());

    let level_area = fixed_intersection(&level_area(&rect), &rect);
    let game_area = fixed_intersection(&game_area(&rect), &rect);
    let right_area = fixed_intersection(&right_area(&rect), &rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(right_area);

    let stats = block("stats");
    let game = block("tetrs");
    let next = block("next");
    let help = block("help");

    let line_vec: Vec<Line> = std::iter::repeat(Line::default())
        .take(FIELD_HEIGHT)
        .collect();
    let mut lines: [Line; FIELD_HEIGHT] = line_vec.try_into().unwrap();

    let next_lines_vec: Vec<Line> = std::iter::repeat(Line::default()).take(6).collect();
    let mut next_lines: [Line; 6] = next_lines_vec.try_into().unwrap();

    let game_paragraph = Paragraph::new(draw_field(state, &mut lines)).block(game);
    let stats_paragraph = Table::new(draw_stats(&state.level))
        .block(stats)
        .widths(&[Constraint::Length(7), Constraint::Length(15)]);
    let next_paragraph = Paragraph::new(draw_next(&state.next, &mut next_lines)).block(next);
    let help_table = Table::new(draw_help())
        .block(help)
        .widths(&[Constraint::Length(8), Constraint::Length(15)]);

    frame.render_widget(stats_paragraph, level_area);
    frame.render_widget(game_paragraph, game_area);
    frame.render_widget(next_paragraph, chunks[0]);
    frame.render_widget(help_table, chunks[1]);
}

fn block(title: &str) -> Block {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
}

fn draw_field<'a>(state: &GameState, rows: &'a mut [Line; FIELD_HEIGHT]) -> Vec<Spans<'a>> {
    if let Some(preview) = &state.preview {
        draw_tetromino(preview, rows, Cell::preview(preview));
    }
    draw_tetromino(&state.current, rows, Cell::normal(&state.current));
    draw_solidified(&state.field, rows);

    rows.iter().map(|x| x.to_spans()).collect()
}

fn draw_tetromino(tetromino: &Tetromino, rows: &mut [Line], cell: Cell) {
    for elem in tetromino.blocks.iter() {
        rows[(tetromino.coords.y + elem.vec.y) as usize].cells
            [(tetromino.coords.x + elem.vec.x) as usize] = cell.clone();
    }
}

fn draw_solidified(field: &Field, rows: &mut [Line; FIELD_HEIGHT]) {
    for (line_index, line) in field.iter().enumerate() {
        for (column_index, square) in line.iter().enumerate() {
            if let Square::Occupied(color) = square {
                rows[line_index].cells[column_index] = Cell {
                    str: "  ",
                    style: Style::default().bg(*color),
                };
            }
        }
    }
}

fn draw_next<'a>(next: &Tetromino, rows: &'a mut [Line; 6]) -> Vec<Spans<'a>> {
    draw_tetromino(next, rows, Cell::normal(next));

    rows.iter().map(|x| x.to_spans()).collect()
}

fn draw_help() -> Vec<Row<'static>> {
    vec![
        Row::new(vec!["", ""]),
        Row::new(vec![" Left", "←"]),
        Row::new(vec![" Right", "→"]),
        Row::new(vec![" Down", "↓"]),
        Row::new(vec![" Rotate", "↑"]),
        Row::new(vec![" Drop", "d, space"]),
        Row::new(vec![" Restart", "r"]),
        Row::new(vec![" Quit", "q, ctrl+c"]),
    ]
}

fn draw_stats(level: &Level) -> Vec<Row<'static>> {
    vec![
        Row::new(vec![String::from(""), String::from("")]),
        Row::new(vec![" Level:".into(), format!("{}", level.current)]),
        Row::new(vec![" Lines:".into(), format!("{}", level.cleared_lines)]),
        Row::new(vec![" Score:".into(), format!("{}", level.score)]),
    ]
}

#[derive(Clone, Debug)]
struct Line {
    pub cells: [Cell; FIELD_WIDTH],
}

impl Default for Line {
    fn default() -> Self {
        let cells: Vec<Cell> = std::iter::repeat(Cell::default())
            .take(FIELD_WIDTH)
            .collect();

        Self {
            cells: cells.try_into().unwrap(),
        }
    }
}

impl Line {
    fn to_spans(&self) -> Spans {
        Spans::from(
            self.cells
                .iter()
                .map(|x| x.to_span())
                .collect::<Vec<Span>>(),
        )
    }
}

#[derive(Debug, Clone)]
struct Cell {
    pub str: &'static str,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            str: "  ",
            style: Default::default(),
        }
    }
}

impl Cell {
    fn to_span(&self) -> Span {
        Span::styled(self.str, self.style)
    }

    fn normal(tetromino: &Tetromino) -> Self {
        Self {
            str: "  ",
            style: Style::default().bg(tetromino.color),
        }
    }

    fn preview(tetromino: &Tetromino) -> Self {
        Self {
            str: "◤◢",
            style: Style::default().fg(tetromino.color),
        }
    }
}

fn draw_menu(frame: &mut tui::Frame<CrosstermBackend<Stdout>>) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Max(u16::MAX),
            Constraint::Percentage(20),
        ])
        .split(frame.size());

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Max(u16::MAX),
            Constraint::Percentage(20),
        ])
        .split(vertical_chunks[1]);

    let rect = horizontal_chunks[1];
    let lines = get_menu_lines(&rect);

    let block = block("tetrs");

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, rect);
}

fn get_menu_lines(rect: &Rect) -> Vec<Spans> {
    let empty_lines = (rect.height / 2).saturating_sub(1);
    let mut lines = vec![Spans::from(""); empty_lines.into()];
    lines.push(Spans::from("Choose a level (0-9)"));

    lines
}
