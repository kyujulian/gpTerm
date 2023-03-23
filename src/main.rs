//logging
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};

//std
use std::{
    // error::Error,
    fs::{File, self},
    io::{self, Read},
};

//tui
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use app::{App, InputMode, MessageType};

use serde::Deserialize;

use toml::ser::SerializeTable::Table;


mod commands;
mod chat_api;
mod api_manager;
mod text_api;
mod app;
mod logging;
mod render;

#[derive(Deserialize)]
struct Config {
    token: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //setup logging

    let log_file = "./log/logfile";
    let request_file = "./log/requests";

    // Log trace level output to file where trace is the default level
    let _handle = logging::set_logging(log_file, request_file);

    //setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    //reading api token
    let user = std::env::var("USER").expect("Can access user environment variable");

    //fix this error handling
    let mut config_file = fs::read_to_string(format!(
        "/home/{}/.config/gpterm/gpterm.toml",
        user.as_str()
    ))
    .unwrap();


    let mut config_string = String::new();
    let configs_toml: Config = toml::from_str(config_file.as_str()).unwrap();

    //create app and run it -> Singleton
    let mut app = App::default();
    app.set_api_manager(configs_toml.token);
    app.set_username(user);

    let res = run_app(&mut terminal, app);

    futures::join!(res);

    //restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // if let Err(err.await) = res {
    //     println!("{:?}", err)
    // }
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    // let mut file = File::create("request.txt")?;
    loop {
        terminal.draw(|f| render::ui(f, &app))?;

        match event::read()? {
            Event::Resize(_, _) => app.update_size(),
            Event::Key(key) => {
                match app.input_mode() {
                    InputMode::Normal => match key.code {
                        KeyCode::Char(':') => {
                            app.set_input_mode(InputMode::Command);
                        }
                        KeyCode::Char('i') => {
                            app.set_input_mode(InputMode::Insert);
                        }
                        KeyCode::Char('k') => {
                            app.scroll_up();
                        }
                        KeyCode::Char('j') => {
                            app.scroll_down();
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::Insert => {
                        match key.code {
                            KeyCode::Enter => {
                                app.push_content(
                                    app.get_username(),
                                    MessageType::Query,
                                    app.get_display_input().drain(..).collect(),
                                );

                                app.set_input_mode(InputMode::Normal);
                                terminal.draw(|f| render::ui(f, &app))?;
                                app.update_input();
                                terminal.draw(|f| render::ui(f, &app))?;

                                app.answer().await;

                                app.scroll_to_bottom();
                            }
                            KeyCode::Char(c) => {
                                app.push_input(c);
                            }
                            KeyCode::Backspace => {
                                app.pop_input();
                            }
                            KeyCode::Esc => {
                                app.set_input_mode(InputMode::Normal);
                            }
                            _ => {}
                        }
                    }
                    InputMode::Command => match key.code {
                        KeyCode::Enter => {
                            app.parse_command();
                            app.execute_command();
                            app.set_input_mode(InputMode::Normal);
                            app.reset_command();
                        }
                        KeyCode::Char(c) => {
                            app.push_command(c);
                        }
                        KeyCode::Backspace => {
                            app.pop_command();
                        }
                        KeyCode::Esc => {
                            app.reset_command();
                            app.set_input_mode(InputMode::Normal);
                        }
                        _ => {}
                    },
                }
            }
            _ => {}
        }
    }
}
