use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame, Terminal,
};
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{collections::HashSet, error::Error, fs::File, io, time::Duration};

#[derive(Parser)]
struct Args {
    /// Number of boxes (default 1, use 2 for two boxes)
    boxes: Option<String>,
}

#[derive(Clone, Copy)]
struct SpecialObject {
    x: u16,
    y: u16,
    color: Color,
}

const SPECIAL_COLORS: [Color; 3] = [Color::Rgb(128, 0, 128), Color::Cyan, Color::Rgb(255, 165, 0)];

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    None,
    Left,
    Right,
    Up,
    Down,
}

struct App {
    fst_x: u16,
    fst_y: u16,
    fst_direction: Direction,
    fst_color: Option<Color>,

    snd_x: u16,
    snd_y: u16,
    snd_direction: Direction,
    snd_color: Option<Color>,

    flash: bool,
    flash_timer: u32,

    specials: Vec<SpecialObject>,
}

impl App {
    fn new() -> Self {
        Self {
            fst_x: 0,
            fst_y: 0,
            fst_direction: Direction::None,
            fst_color: None,

            snd_x: 0,
            snd_y: 0,
            snd_direction: Direction::None,
            snd_color: None,

            flash: false,
            flash_timer: 0,

            specials: vec![],
        }
    }

    fn set_initial_positions(&mut self, width: u16, height: u16) {
        self.snd_x = width.saturating_sub(2);
        self.snd_y = height.saturating_sub(2);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("kli.log").unwrap(),
    );

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let initial_size = terminal.size()?;

    let mut app = App::new();
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

        if app.flash_timer > 0 {
            app.flash_timer -= 1;
            if app.flash_timer == 0 {
                app.flash = false;
            }
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Left => {
                        if app.fst_direction == Direction::Right {
                            app.fst_direction = Direction::None;
                        } else {
                            let can_move = app.fst_x > 0 && !(app.fst_x.saturating_sub(app.snd_x) <= 2 && app.fst_y == app.snd_y);
                            if can_move {
                                app.fst_direction = Direction::Left;
                            }
                        }
                    }
                    KeyCode::Right => {
                        if app.fst_direction == Direction::Left {
                            app.fst_direction = Direction::None;
                        } else {
                            let can_move = app.fst_x + 2 < size.width && !(app.snd_x.saturating_sub(app.fst_x) <= 2 && app.fst_y == app.snd_y);
                            if can_move {
                                app.fst_direction = Direction::Right;
                            }
                        }
                    }
                    KeyCode::Up => {
                        if app.fst_direction == Direction::Down {
                            app.fst_direction = Direction::None;
                        } else {
                            let can_move = app.fst_y > 0 && !(app.fst_y.saturating_sub(app.snd_y) <= 2 && app.fst_x == app.snd_x);
                            if can_move {
                                app.fst_direction = Direction::Up;
                            }
                        }
                    }
                    KeyCode::Down => {
                        if app.fst_direction == Direction::Up {
                            app.fst_direction = Direction::None;
                        } else {
                            let can_move = app.fst_y + 2 < size.height && !(app.snd_y.saturating_sub(app.fst_y) <= 2 && app.fst_x == app.snd_x);
                            if can_move {
                                app.fst_direction = Direction::Down;
                            }
                        }
                    }
                    KeyCode::Char('a') | KeyCode::Char('4') => {
                        if app.snd_direction == Direction::Right {
                            app.snd_direction = Direction::None;
                        } else {
                            let can_move = app.snd_x > 0 && !(app.snd_x.saturating_sub(app.fst_x) <= 2 && app.snd_y == app.fst_y);
                            if can_move {
                                app.snd_direction = Direction::Left;
                            }
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Char('6') => {
                        if app.snd_direction == Direction::Left {
                            app.snd_direction = Direction::None;
                        } else {
                            let can_move = app.snd_x + 2 < size.width && !(app.fst_x.saturating_sub(app.snd_x) <= 2 && app.snd_y == app.fst_y);
                            if can_move {
                                app.snd_direction = Direction::Right;
                            }
                        }
                    }
                    KeyCode::Char('w') | KeyCode::Char('8') => {
                        if app.snd_direction == Direction::Down {
                            app.snd_direction = Direction::None;
                        } else {
                            let can_move = app.snd_y > 0 && !(app.snd_y.saturating_sub(app.fst_y) <= 2 && app.snd_x == app.fst_x);
                            if can_move {
                                app.snd_direction = Direction::Up;
                            }
                        }
                    }
                    KeyCode::Char('s') | KeyCode::Char('2') => {
                        if app.snd_direction == Direction::Up {
                            app.snd_direction = Direction::None;
                        } else {
                            let can_move = app.snd_y + 2 < size.height && !(app.fst_y.saturating_sub(app.snd_y) <= 2 && app.snd_x == app.fst_x);
                            if can_move {
                                app.snd_direction = Direction::Down;
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            // no event, continue moving in current fst_direction
            let mut new_fst_x = app.fst_x;
            let mut new_fst_y = app.fst_y;
            match app.fst_direction {
                Direction::Left => {
                    if new_fst_x > 0 {
                        new_fst_x -= 1;
                        if new_fst_x == 0 {
                            app.fst_color = new_color_by_border(app.fst_direction);
                            app.fst_direction = Direction::None;
                        }
                    }
                }
                Direction::Right => {
                    if new_fst_x + 2 < size.width {
                        new_fst_x += 1;
                        if new_fst_x == size.width - 2 {
                            app.fst_color = new_color_by_border(app.fst_direction);
                            app.fst_direction = Direction::None;
                        }
                    }
                }
                Direction::Up => {
                    if new_fst_y > 0 {
                        new_fst_y -= 1;
                        if new_fst_y == 0 {
                            app.fst_color = new_color_by_border(app.fst_direction);
                            app.fst_direction = Direction::None;
                        }
                    }
                }
                Direction::Down => {
                    if new_fst_y + 2 < size.height {
                        new_fst_y += 1;
                        if new_fst_y == size.height - 2 {
                            app.fst_color = new_color_by_border(app.fst_direction);
                            app.fst_direction = Direction::None;
                        }
                    }
                }
                Direction::None => {}
            }

            let mut new_snd_x = app.snd_x;
            let mut new_snd_y = app.snd_y;
            match app.snd_direction {
                Direction::Left => {
                    if new_snd_x > 0 {
                        new_snd_x -= 1;
                        if new_snd_x == 0 {
                            app.snd_color = new_color_by_border(app.snd_direction);
                            app.snd_direction = Direction::None;
                        }
                    }
                }
                Direction::Right => {
                    if new_snd_x + 2 < size.width {
                        new_snd_x += 1;
                        if new_snd_x == size.width - 2 {
                            app.snd_color = new_color_by_border(app.snd_direction);
                            app.snd_direction = Direction::None;
                        }
                    }
                }
                Direction::Up => {
                    if new_snd_y > 0 {
                        new_snd_y -= 1;
                        if new_snd_y == 0 {
                            app.snd_color = new_color_by_border(app.snd_direction);
                            app.snd_direction = Direction::None;
                        }
                    }
                }
                Direction::Down => {
                    if new_snd_y + 2 < size.height {
                        new_snd_y += 1;
                        if new_snd_y == size.height - 2 {
                            app.snd_color = new_color_by_border(app.snd_direction);
                            app.snd_direction = Direction::None;
                        }
                    }
                }
                Direction::None => {}
            }

            // check specials
            let mut to_remove = HashSet::new();
            for (i, special) in app.specials.iter().enumerate() {
                if new_fst_x <= special.x && special.x < new_fst_x + 2 && new_fst_y <= special.y && special.y < new_fst_y + 2 {
                    app.fst_color = Some(special.color);
                    to_remove.insert(i);
                }
                if new_snd_x <= special.x && special.x < new_snd_x + 2 && new_snd_y <= special.y && special.y < new_snd_y + 2 {
                    app.snd_color = Some(special.color);
                    to_remove.insert(i);
                }
            }
            let mut to_remove: Vec<usize> = to_remove.into_iter().collect();
            to_remove.sort_unstable_by(|a, b| b.cmp(a));
            for i in to_remove {
                app.specials.remove(i);
            }

            // check for collision
            let collision = new_fst_x < new_snd_x + 2
                && new_fst_x + 2 > new_snd_x
                && new_fst_y < new_snd_y + 2
                && new_fst_y + 2 > new_snd_y;

            if collision {
                // drop special if same color
                if app.fst_color == app.snd_color {
                    let current = app.fst_color.unwrap_or(Color::Black);
                    let index = SPECIAL_COLORS.iter().position(|&c| c == current).unwrap_or(0);
                    let next_color = SPECIAL_COLORS.get(index + 1).copied().unwrap_or(SPECIAL_COLORS[0]);
                    let mut rng = rand::thread_rng();
                    let x = rng.gen_range(0..size.width.saturating_sub(2));
                    let y = rng.gen_range(0..size.height.saturating_sub(2));
                    app.specials.push(SpecialObject { x, y, color: next_color });
                }
                // swap fst_colors
                let temp = app.fst_color;
                app.fst_color = app.snd_color;
                app.snd_color = temp;
                // if both have same fst_color now, uncolor
                if app.fst_color == app.snd_color && app.fst_color.is_some() {
                    app.fst_color = None;
                    app.snd_color = None;
                }
                app.fst_direction = Direction::None;
                app.snd_direction = Direction::None;
                // flash screen
                app.flash = true;
                app.flash_timer = 5;
            } else {
                // no collision, move
                app.fst_x = new_fst_x;
                app.fst_y = new_fst_y;
                app.snd_x = new_snd_x;
                app.snd_y = new_snd_y;
            }
        }
    }
}

fn new_color_by_border(dir: Direction) -> Option<Color> {
    match dir {
        Direction::Left => Some(Color::Red),
        Direction::Right => Some(Color::Blue),
        Direction::Up => Some(Color::Green),
        Direction::Down => Some(Color::Yellow),
        _ => None,
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let mut block = Block::default();
    if app.flash {
        block = block.style(Style::default().bg(Color::White));
    }
    f.render_widget(block, size);

    let mut square = Block::default().borders(Borders::ALL);
    if let Some(fst_color) = app.fst_color {
        square = square.style(Style::default().bg(fst_color));
    }

    let fst_area = ratatui::layout::Rect::new(app.fst_x, app.fst_y, 2, 2);
    f.render_widget(square, fst_area);

    let mut snd_square = Block::default().borders(Borders::ALL);
    if let Some(snd_color) = app.snd_color {
        snd_square = snd_square.style(Style::default().bg(snd_color));
    }
    let snd_area = ratatui::layout::Rect::new(app.snd_x, app.snd_y, 2, 2);
    f.render_widget(snd_square, snd_area);

    for special in &app.specials {
        let special_block = Block::default().borders(Borders::ALL).style(Style::default().bg(special.color));
        let area = ratatui::layout::Rect::new(special.x, special.y, 2, 2);
        f.render_widget(special_block, area);
    }
}
