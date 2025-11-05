use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Vec<bool>>,
    cursor_x: usize,
    cursor_y: usize,
    autotoggle: bool,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        Grid {
            width,
            height,
            cells: vec![vec![false; width]; height],
            cursor_x: 0,
            cursor_y: 0,
            autotoggle: false,
        }
    }

    fn toggle_current(&mut self) {
        self.cells[self.cursor_y][self.cursor_x] = !self.cells[self.cursor_y][self.cursor_x];
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        let new_x = (self.cursor_x as isize + dx)
            .max(0)
            .min(self.width as isize - 1) as usize;
        let new_y = (self.cursor_y as isize + dy)
            .max(0)
            .min(self.height as isize - 1) as usize;
        self.cursor_x = new_x;
        self.cursor_y = new_y;
        if self.autotoggle {
            self.toggle_current();
        }
    }
}

fn create_grid_text(grid: &Grid) -> Text<'static> {
    let mut lines = Vec::new();
    for (y, row) in grid.cells.iter().enumerate() {
        let mut spans = Vec::new();
        for (x, &filled) in row.iter().enumerate() {
            let is_cursor = x == grid.cursor_x && y == grid.cursor_y;
            let ch = if filled { '█' } else { '░' };
            let style = if is_cursor {
                Style::default().fg(Color::Yellow).bg(Color::Blue)
            } else {
                Style::default()
            };
            spans.push(Span::styled(ch.to_string(), style));
        }
        lines.push(Line::from(spans));
    }
    Text::from(lines)
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let size = terminal.size()?;
    let grid_width = size.width as usize;
    let grid_height = (size.height - 2) as usize; // Reserve space for borders

    let mut grid = Grid::new(grid_width, grid_height);

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let title = if grid.autotoggle {
                "Draw Mode (Ctrl+Space to toggle)"
            } else {
                "Normal Mode (Ctrl+Space to toggle)"
            };
            let block = Block::default().borders(Borders::ALL).title(title);
            let inner_area = block.inner(size);
            f.render_widget(block, size);
            let text = create_grid_text(&grid);
            let paragraph = Paragraph::new(text);
            f.render_widget(paragraph, inner_area);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(' ') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        grid.autotoggle = !grid.autotoggle;
                    } else {
                        grid.toggle_current();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => grid.move_cursor(0, -1),
                KeyCode::Down | KeyCode::Char('j') => grid.move_cursor(0, 1),
                KeyCode::Left | KeyCode::Char('h') => grid.move_cursor(-1, 0),
                KeyCode::Right | KeyCode::Char('l') => grid.move_cursor(1, 0),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
