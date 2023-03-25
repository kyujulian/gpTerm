
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


use crate::app::{App, InputMode, CommandStatus};

pub fn ui<B: Backend> (f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(9),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
            Constraint::Length(1),
        ].as_ref()
    )
    .split(f.size());




    //TODO support changing models
    let (msg, style) = 
        (
            vec![
                Span::styled("GPT-3.5",Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)),
                // Span::raw(" GPT-4 "),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
       );


    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let models = Paragraph::new(text).block(Block::default()
        .borders(Borders::TOP)
        .title_alignment(Alignment::Center)
        .title("  Models  "))
        .alignment(Alignment::Center);
    f.render_widget(models, chunks[0]);

    let command = Paragraph::new(app.get_command())
        .style(match app.command_status() {
            CommandStatus::Error => {
                Style::default().bg(Color::Black).fg(Color::LightRed)
            }
            _ => Style::default().bg(Color::Black).fg(Color::White),
    })
    .block(Block::default());

    let input = Paragraph::new(app.get_display_input())
        .style(match app.input_mode() {
            InputMode::Normal => Style::default(),
            InputMode::Insert => Style::default().fg(Color::Blue),
            InputMode::Command => Style::default().fg(Color::Yellow),
    })
    .block(Block::default().borders(Borders::TOP).title(" Input ")).wrap(Wrap{trim: false});
    f.render_widget(input, chunks[2]);

    match app.input_mode() {
        InputMode::Normal => {
            if let crate::app::CommandStatus::Error = app.command_status(){
                f.render_widget(command, chunks[3]);
            }
        } //Hide the cursor. 'Frame does this by default.
        InputMode::Insert => {
            f.set_cursor(
                chunks[2].x + app.get_display_input().width() as u16,
                chunks[2].y + 1,
            )
        }
        InputMode::Command => {

            f.render_widget(command, chunks[3]);
            f.set_cursor(
                chunks[3].x + app.get_command().width() as u16,
                chunks[3].y,
            )
        }

    }
    let messages: Vec<Spans> = app
        .get_content();

    let messages = 
        Paragraph::new(messages).block(Block::default()
        .borders(Borders::TOP)
        .title_alignment(Alignment::Center)
        .title(" Chat "))
        .wrap(Wrap{trim: false})
        .scroll((app.get_scroll() as u16,0));
    f.render_widget(messages,chunks[1]);
}

