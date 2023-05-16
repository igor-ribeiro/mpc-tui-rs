extern crate ncurses;

use std::ops::{Add, Sub};

use ncurses::*;

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

enum Mode {
    Normal,
    Insert,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Pos(i32, i32);

impl Add for Pos {
    type Output = Pos;
    fn add(self, rhs: Self) -> Pos {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub for Pos {
    type Output = Pos;
    fn sub(self, rhs: Self) -> Pos {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

#[derive(Debug, Clone)]
enum ElementKind {
    Input { label: String, value: String },
    Button { label: String, active: bool },
    Title(String),
}

#[derive(Debug, Clone)]
struct Element {
    kind: ElementKind,
    pos: Pos,
    width: i32,
    focusable: bool,
}

#[derive(Debug, Copy, Clone)]
struct Screen {
    y: i32,
    x: i32,
    width: i32,
    height: i32,
}

struct App {
    key: Option<i32>,
    focus: Option<&'static str>,
    mode: Mode,
    screen: Screen,
    cursor: Pos,
    elements: Vec<Element>,
    focusabled_elements: Vec<Element>,
    render_cursor: Pos,
    actions: Vec<&'static str>,
}

impl App {
    fn new() -> Self {
        Self {
            key: None,
            focus: None,
            mode: Mode::Normal,
            screen: Screen {
                y: 0,
                x: 0,
                width: 40,
                height: 16,
            },
            cursor: Pos(0, 0),
            elements: Vec::new(),
            focusabled_elements: Vec::new(),
            render_cursor: Pos(0, 0),
            actions: Vec::new(),
        }
    }

    fn render_container(&mut self) {
        let x = self.screen.x;

        mv(0, x);
        addch(ACS_ULCORNER());
        for _ in 0..self.screen.width {
            addch(ACS_HLINE());
        }
        addch(ACS_URCORNER());

        mv(self.screen.height as i32, x);
        addch(ACS_LLCORNER());
        for _ in 0..self.screen.width {
            addch(ACS_HLINE());
        }
        addch(ACS_LRCORNER());

        for row in 1..self.screen.height {
            mv(row as i32, x);
            addch(ACS_VLINE());
        }
        for row in 1..self.screen.height {
            mv(row as i32, x + self.screen.width as i32 + 1);
            addch(ACS_VLINE());
        }
    }

    fn render_actions(&mut self, active: Option<usize>) {
        let count = self.actions.len() as i32;
        let actions = self.actions.clone();
        let size = (self.screen.width / count) as usize;

        for (col, label) in actions.iter().enumerate() {
            let hint = &format!("{:^w$}", col + 1, w = size);

            let y = self.screen.height as i32;
            let x = self.screen.x + (size * (col)) as i32;

            let active = match active {
                Some(index) if (index - 1) == col => true,
                _ => false,
            };

            self.render_cursor = Pos(x + 1, y - 1);
            self.create_button(label, Some(size - 2), active);

            mv(y + 1, x + 1);
            attron(A_DIM());
            addstr(hint);
            attroff(A_DIM());
            mv(self.screen.y, self.screen.x);
        }
    }

    fn get_key_char(&mut self) -> Option<char> {
        self.key.take().map(|k| k as u8 as char)
    }

    fn reset(&mut self) {
        self.elements = Vec::new();
        self.render_cursor = Pos(self.screen.x + 1, 1);
    }

    fn create_title(&mut self, title: &str) -> Element {
        let width = self.screen.width as usize;
        let size = title.len().min(width);
        let trimmed_title = if size == width {
            // width - 7 because:
            // = title... =
            // 12     34567
            format!("{}...", &title[..(width - 7)])
        } else {
            title.to_string()
        };

        let title = &format!("{:=^left$}", format!(" {} ", trimmed_title), left = width);
        let size = title.len();

        let pos = Pos(self.render_cursor.0, self.render_cursor.1);

        mv(self.render_cursor.1, self.render_cursor.0);
        addstr(&title);

        self.next_row();

        let element = Element {
            kind: ElementKind::Title(title.to_string()),
            pos,
            width: size as i32,
            focusable: false,
        };

        self.elements.push(element.clone());

        element.clone()
    }

    fn move_render_cursor(&mut self, x: i32, y: i32) -> Pos {
        self.render_cursor.0 += x;
        self.render_cursor.1 += y;

        self.render_cursor.clone()
    }

    fn create_input(&mut self, label: &str, value: &str, width: Option<usize>) -> Element {
        let input_label = format!("{}: ", label);
        let size = input_label.len();
        let width = width.unwrap_or(size + 1).max(size + 1);

        let input_value = format!("{:<width$}", value, width = width);
        let input_size = format!("{}{}", input_label, input_value).len() as i32;

        let pos = Pos(self.render_cursor.0, self.render_cursor.1);

        let pair = if pos == self.cursor {
            HIGHLIGHT_PAIR
        } else {
            REGULAR_PAIR
        };

        mv(self.render_cursor.1, self.render_cursor.0);
        addstr(&input_label);

        attron(COLOR_PAIR(pair));
        addstr(&input_value);
        attroff(COLOR_PAIR(pair));

        self.move_render_cursor(input_size, 0);

        let element = Element {
            kind: ElementKind::Input {
                label: label.to_string(),
                value: value.to_string(),
            },
            pos,
            width: input_size,
            focusable: true,
        };

        self.elements.push(element.clone());

        element
    }

    fn create_button(&mut self, label: &str, size: Option<usize>, active: bool) -> Element {
        let button_label = format!("[{:^w$}]", label, w = size.unwrap_or(label.len() + 2));
        let size = button_label.len() as i32;

        let element = Element {
            kind: ElementKind::Button {
                label: button_label.to_string(),
                active,
            },
            pos: self.render_cursor.clone(),
            width: size,
            focusable: false,
        };

        let pair = if active {
            COLOR_PAIR(HIGHLIGHT_PAIR)
        } else {
            COLOR_PAIR(REGULAR_PAIR)
        };

        mv(self.render_cursor.1, self.render_cursor.0);
        attron(pair);
        addstr(&button_label);
        attroff(pair);

        self.move_render_cursor(element.width, 0);

        self.elements.push(element.clone());

        element
    }

    fn next_row(&mut self) {
        self.render_cursor = Pos(self.screen.x + 1, self.render_cursor.1 + 1);
    }

    fn move_up(&mut self) {
        self.focusabled_elements = self
            .elements
            .clone()
            .into_iter()
            .filter(|el| el.pos.1 < self.cursor.1 && el.focusable)
            .rev()
            .collect::<Vec<_>>();
    }

    fn move_down(&mut self) {
        self.focusabled_elements = self
            .elements
            .clone()
            .into_iter()
            .filter(|el| el.pos.1 > self.cursor.1 && el.focusable)
            .collect::<Vec<_>>();
    }

    fn move_left(&mut self) {
        self.focusabled_elements = self
            .elements
            .clone()
            .into_iter()
            .filter(|el| el.pos.1 == self.cursor.1 && el.pos.0 < self.cursor.0)
            .collect::<Vec<_>>();
    }

    fn move_right(&mut self) {
        self.focusabled_elements = self
            .elements
            .clone()
            .into_iter()
            .filter(|el| el.pos.1 == self.cursor.1 && el.pos.0 > self.cursor.0 && el.focusable)
            .collect::<Vec<_>>();
    }

    fn update_focus(&mut self) {
        if self.focusabled_elements.is_empty() {
            return;
        }

        self.cursor = self.focusabled_elements[0].pos.clone();
    }

    fn focus_element(&mut self, index: usize) {
        let element = self.elements.iter().find(|el| el.focusable);

        if let Some(element) = element {
            self.cursor = element.pos.clone();
        }
    }
}

fn main() {
    initscr();
    keypad(stdscr(), true);
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    timeout(16);

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    let mut quit = false;
    let mut app = App::new();
    let mut notification = String::new();
    let mut started = false;
    let mut active_action: Option<usize> = None;

    while !quit {
        erase();

        let mut width = 0;
        let mut height = 0;
        getmaxyx(stdscr(), &mut height, &mut width);

        app.screen.x = (width - app.screen.width) / 2;
        app.actions = vec!["TODO", "DONE"];

        app.render_container();

        app.render_cursor = Pos(app.screen.x + 1, 1);

        if !started {
            app.cursor = app.render_cursor.clone();
        }

        app.create_title("Play/Record");
        app.create_input("Seq", "1-(unused)", Some(20));
        app.create_input("BPM", "120.0", None);
        //
        // app.create_input("Name", "(untitled)", Some(20));
        // app.next_row();
        // app.create_title("Play/Record");
        //
        // app.create_input("Size", "10", None);

        if !started {
            app.focus_element(0);
            app.update_focus();
        }

        app.render_actions(active_action);
        // app.render_elements();

        if let Some(key) = app.get_key_char() {
            match key {
                'k' => app.move_up(),
                'j' => app.move_down(),
                'l' => app.move_right(),
                'h' => app.move_left(),
                k => {
                    let digit = key.to_digit(16);

                    if let Some(number) = digit {
                        let number = number as usize;
                        if number <= app.actions.len() {
                            active_action = Some(number);
                        }
                    } else {
                        notification.clear();
                    }

                    app.key = Some(key as u8 as i32);
                }
            }

            if 'q' == key {
                quit = true;
            }
        }

        mv(app.screen.height + 2, app.screen.x);
        notification.push_str("test");
        addstr(&notification);

        app.update_focus();

        refresh();

        let key = getch();

        if key != ERR {
            app.key = Some(key);
        }

        app.reset();
        notification.clear();
        started = true;
    }

    endwin();
}
