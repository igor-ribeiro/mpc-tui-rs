extern crate ncurses;

use mpc_tui_rs::*;
use ncurses::*;

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
        app.create_input("Seq", "1-(unused)", None);
        app.create_input("BPM", "120.0", None);

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
