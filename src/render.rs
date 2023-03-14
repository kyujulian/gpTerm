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


use crate::app::{App, InputMode};

pub fn ui<B: Backend> (f: &mut Frame<B>, app: &App) {
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

