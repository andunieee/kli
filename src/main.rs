use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders},
    Frame, Terminal,
};
use std::{error::Error, io};

#[derive(Parser)]
struct Args {
    /// Number of boxes (default 1, use 2 for two boxes)
    boxes: Option<String>,
}

struct App {
    x: u16,
    y: u16,
    second_x: u16,
    second_y: u16,
    has_second: bool,
}

impl App {
    fn new(has_second: bool) -> Self {
        Self {
            x: 0,
            y: 0,
            second_x: 0,
            second_y: 0,
            has_second,
        }
    }

    fn set_initial_positions(&mut self, width: u16, height: u16) {
        if self.has_second {
            self.second_x = width.saturating_sub(2);
            self.second_y = height.saturating_sub(2);
        }
    }

    fn move_left(&mut self) {
        if self.x > 0 {
            self.x -= 1;
        }
    }

    fn move_right(&mut self, max_x: u16) {
        if self.x + 2 < max_x {
            self.x += 1;
        }
    }

    fn move_up(&mut self) {
        if self.y > 0 {
            self.y -= 1;
        }
    }

    fn move_down(&mut self, max_y: u16) {
        if self.y + 2 < max_y {
            self.y += 1;
        }
    }

    fn move_second_left(&mut self) {
        if self.has_second && self.second_x > 0 {
            self.second_x -= 1;
        }
    }

    fn move_second_right(&mut self, max_x: u16) {
        if self.has_second && self.second_x + 2 < max_x {
            self.second_x += 1;
        }
    }

    fn move_second_up(&mut self) {
        if self.has_second && self.second_y > 0 {
            self.second_y -= 1;
        }
    }

    fn move_second_down(&mut self, max_y: u16) {
        if self.has_second && self.second_y + 2 < max_y {
            self.second_y += 1;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let has_second = args.boxes.as_deref() == Some("2");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let initial_size = terminal.size()?;

    let mut app = App::new(has_second);
    app.set_initial_positions(initial_size.width, initial_size.height);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        let size = terminal.size()?;
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Left => app.move_left(),
                KeyCode::Right => app.move_right(size.width),
                KeyCode::Up => app.move_up(),
                KeyCode::Down => app.move_down(size.height),
                KeyCode::Char('a') => app.move_second_left(),
                KeyCode::Char('d') => app.move_second_right(size.width),
                KeyCode::Char('w') => app.move_second_up(),
                KeyCode::Char('s') => app.move_second_down(size.height),
                KeyCode::Char('4') => app.move_second_left(),
                KeyCode::Char('6') => app.move_second_right(size.width),
                KeyCode::Char('8') => app.move_second_up(),
                KeyCode::Char('2') => app.move_second_down(size.height),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();
    let block = Block::default();
    f.render_widget(block, size);

    let square = Block::default().borders(Borders::ALL);
    let area = ratatui::layout::Rect::new(app.x, app.y, 2, 2);
    f.render_widget(square, area);

    if app.has_second {
        let second_square = Block::default().borders(Borders::ALL);
        let second_area = ratatui::layout::Rect::new(app.second_x, app.second_y, 2, 2);
        f.render_widget(second_square, second_area);
    }
}
