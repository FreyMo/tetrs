use std::{
    cmp::{max, min},
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
    state::{Field, GameMode, GameState, Square, FIELD_HEIGHT, FIELD_WIDTH},
    tetromino::Tetromino,
};

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Default for Ui {
    fn default() -> Self {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).unwrap();

        Self { terminal }
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
    pub fn draw(&mut self, mode: &GameMode) {
        self.terminal.draw(|frame| draw_frame(mode, frame)).unwrap();
    }
}

fn draw_frame(mode: &GameMode, frame: &mut tui::Frame<CrosstermBackend<Stdout>>) {
    match mode {
        GameMode::Menu(_) => draw_menu(frame),
        GameMode::Running(running) => draw_tetrs(&running.state, frame),
        GameMode::Finished(finished) => draw_tetrs(&finished.state, frame),
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

pub fn fixed_intersection(left: &Rect, other: &Rect) -> Rect {
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

    let stats = Block::default()
        .title("stats")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center);
    let game = Block::default()
        .title("tetrs")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center);
    let next = Block::default()
        .title("next")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center);
    let help = Block::default()
        .title("help")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center);

    let line_vec: Vec<Line> = std::iter::repeat(Line::default())
        .take(FIELD_HEIGHT)
        .collect();
    let mut lines: [Line; FIELD_HEIGHT] = line_vec.try_into().unwrap();

    let next_lines_vec: Vec<Line> = std::iter::repeat(Line::default()).take(6).collect();
    let mut next_lines: [Line; 6] = next_lines_vec.try_into().unwrap();

    let game_paragraph = Paragraph::new(draw_field(state, &mut lines)).block(game);
    let stats_paragraph = Table::new(draw_level(&state.level))
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

fn draw_field<'a>(state: &GameState, rows: &'a mut [Line; FIELD_HEIGHT]) -> Vec<Spans<'a>> {
    if let Some(preview) = &state.preview {
        draw_tetromino(preview, rows, true);
    }
    draw_tetromino(&state.current, rows, false);
    draw_solidified(&state.field, rows);

    rows.iter().map(|x| x.to_spans()).collect()
}

fn draw_tetromino(tetromino: &Tetromino, rows: &mut [Line], preview: bool) {
    for elem in tetromino.blocks.iter() {
        let cell = match preview {
            true => Cell {
                str: "◤◢",
                style: Style::default().fg(tetromino.color),
            },
            false => Cell {
                str: "  ",
                style: Style::default().bg(tetromino.color),
            },
        };

        rows[(tetromino.coords.y + elem.vec.y) as usize].cells
            [(tetromino.coords.x + elem.vec.x) as usize] = cell;
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
    draw_tetromino(next, rows, false);

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

fn draw_level<'a>(level: &Level) -> Vec<Row<'static>> {
    vec![
        Row::new(vec![String::from(""), String::from("")]),
        Row::new(vec![" Level:".into(), format!("{}", level.current)]),
        Row::new(vec![" Lines:".into(), format!("{}", level.cleared_lines)]),
        Row::new(vec![" Score:".into(), format!("{}", level.score)]),
    ]
}

#[derive(Clone, Debug)]
struct Line {
    pub cells: [Cell<'static>; FIELD_WIDTH],
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
struct Cell<'a> {
    pub str: &'a str,
    pub style: Style,
}

impl Default for Cell<'static> {
    fn default() -> Self {
        Self {
            str: "  ",
            style: Default::default(),
        }
    }
}

impl<'a> Cell<'a> {
    fn to_span(&self) -> Span<'a> {
        Span::styled(self.str, self.style)
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

    let block = Block::default()
        .title("tetrs")
        .borders(Borders::ALL)
        .title_alignment(tui::layout::Alignment::Center);

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