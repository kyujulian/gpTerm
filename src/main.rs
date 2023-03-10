//logging
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};



use crossterm:: {
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen
    }
};

use std::{error::Error, io::{self, Write, Read}, fs::File};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};

use unicode_width::UnicodeWidthStr;

use app::{App, InputMode, MessageType};

mod logging;
mod api;
mod app;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{

    
    //setup logging
    //*****************LOGGING************************************

    let log_file = "./log/logfile";
    let request_file = "./log/requests";

    // Log trace level output to file where trace is the default level
    let _handle = logging::set_logging(log_file, request_file);


    //*****************LOGGING************************************
    //setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    //reading api token
    let user = std::env::var("USER").expect("Can access user environment variable");

    let mut config_file = File::open(
        format!("/home/{}/.config/.gpterm/gpterm.conf",
                    user.as_str()))
        .unwrap();

    let mut token = String::new();

    config_file.read_to_string(&mut token).unwrap();
    token = token.trim().to_string();

    //create app and run it -> Singleton
    let mut app = App::default();
    app.set_handler(token);
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

async fn run_app<B: Backend>(terminal : &mut Terminal<B>, mut app: App) -> io::Result<()> {

    // let mut file = File::create("request.txt")?;
    loop {

        terminal.draw(|f| ui(f, &app))?;

        match event::read()? {
            Event::Resize(_, _) => {
                app.update_size()
            }
            Event::Key(key) => {

                match app.input_mode() {
                    InputMode::Normal => match key.code {
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
                                    app.get_display_input().drain(..).collect()
                                );

                                terminal.draw(|f| ui(f, &app))?;
                                app.update_input();
                                terminal.draw(|f| ui(f, &app))?;

                                app.answer().await;

                                let thing = app.get_call();

                                // file.write_all(format!("{:#?}",thing).as_bytes())?;

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
                    _ => {}
                }

            }
            _ => {}
        }
    }
}

fn ui<B: Backend> (f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ].as_ref()
    )
    .split(f.size());




    //TODO support changing models
    let (msg, style) = 
        (
            vec![
                Span::styled(" Davinci ",Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)),
                Span::raw(" Mark "),
                Span::raw(" Dall-e "),
                Span::raw(" Curie "),
                Span::raw(" Ada "),
                // Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                // Span::raw(" to exit, "),
                // Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                // Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
       );


    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text).block(Block::default()
        .borders(Borders::TOP)
        .title_alignment(Alignment::Center)
        .title("  models  "))
        .alignment(Alignment::Center);
    f.render_widget(help_message, chunks[0]);


    let input = Paragraph::new(app.get_display_input())
        .style(match app.input_mode() {
            InputMode::Normal => Style::default(),
            InputMode::Insert => Style::default().fg(Color::Blue),
            InputMode::Command => Style::default().fg(Color::Yellow),
    })
    .block(Block::default().borders(Borders::TOP).title("Query")).wrap(Wrap{trim: false});
    f.render_widget(input, chunks[2]);

    match app.input_mode() {
        InputMode::Normal => {} //Hide the cursor. 'Frame does this by default.
        InputMode::Insert => {
            f.set_cursor(
                chunks[2].x + app.get_display_input().width() as u16,
                chunks[2].y + 1,
            )
        }
        InputMode::Command => {
            f.set_cursor(
                chunks[2].x + app.get_display_input().width() as u16,
                chunks[2].y + 1,
            )
        }

    }
    let messages: Vec<Spans> = app
        .get_content();

    let messages = 
        Paragraph::new(messages).block(Block::default()
        .borders(Borders::TOP)
        .title("chat"))
        .alignment(Alignment::Left)
        .wrap(Wrap{trim: false})
        .scroll((app.get_scroll() as u16,0));
    f.render_widget(messages,chunks[1]);
}





