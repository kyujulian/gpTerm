//logging
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};

use futures::join;
use std::{future::Future, fmt::Display, };

use tui::{style::{Style, Modifier, Color}, 
    text::{Text, Span,Spans}};

use terminal_size::{self,Width,Height};


use crate::{text_api::{TextApi, TextApiCall}, api_manager::{ApiManager, CallType}};
use crate::text_api::TextApiResponse;

fn get_terminal_sizes() -> (u16, u16) {
    let size = terminal_size::terminal_size();
    if let Some((Width(w), Height(h))) = size {
        return (w - 2, h); //tui borders
    } else {
        error!("Couldn't get terminal size!");
        panic!();
    }
}





#[derive(Clone,Copy)]
pub enum InputMode{
    Normal,
    Insert, 
    Command
}


#[derive(Clone,  Debug)]
pub enum MessageType{
    Query,
    Answer
}

#[derive(Clone,Debug)]
pub struct DisplayMessage {
    sender: String,
    body: String,
    message_type: MessageType,
}


impl DisplayMessage {

    pub fn from(sender: String, body: String, message_type: MessageType)
        -> DisplayMessage {
        return DisplayMessage{
            sender, body, message_type
        }
    }
    pub fn get_body(&self) -> &String {
        return &self.body;
    }

    fn get_body_lines(&self, width: u16) -> usize {
        // let normal_line_count: usize = self.body.chars().filter(|x| x == '\n').count();
        let normal_line_count: usize = self.body.lines().count();
        let wrapped_line_count: usize= self.body.lines()
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|x| x.chars().count() / width as usize )
            .sum();

        return normal_line_count + wrapped_line_count; //lines that separate users
    }

    pub fn error(arg: String) -> DisplayMessage {
        DisplayMessage::from("YAS - your average system".to_string(),
            arg, MessageType::Answer)
    }
}

pub enum CommandStatus {
    Okay,
    Error
}
///App holds the state of the application
pub struct App {

    username: String,
    ///current value of the input box
    should_end: bool,

    ///current value displayed on the input box
    display_input: String,
    ///current value of the input box
    internal_input: String,
    ///Current input mode,
    input_mode: InputMode,
    ///History of recorded messages
    content: Vec<DisplayMessage>,
    ///Scrolling tracker
    scroll: usize,
    max_offset: usize,
    ///Command
    command: String,
    command_status: CommandStatus,
    ///Terminal size
    pub size: (u16, u16),
    ///Client to communicate with API
    api_manager: Option<ApiManager>,
}


impl App {


    pub fn command_active(&self) -> bool {
        if self.command == String::from("") {
            return false
        }
        return true
    }

    // pub fn get_call(&self) -> &TextApiCall {
    //     return self.api_manager.as_ref().unwrap().call.as_ref().unwrap()
    // }
    //
    // pub fn get_response(&self) -> &TextApiResponse {
    //     return self.api_manager.as_ref().unwrap().response.as_ref().unwrap()
    // }
    pub fn set_api_manager(&mut self, token: String) {
        self.api_manager = Some(ApiManager::new(token))
    }

    pub fn update_input(&mut self) {
        self.internal_input = String::from(&self.display_input);
        self.display_input = String::new();
    }
    pub async fn answer(&mut self) {

        let query = self.get_input();
        let output = self.api_manager.as_mut().unwrap().answer_from(
            query,
            CallType::Chat
        ).await;

        self.push_answer(output);
    }

    pub fn get_max_offset(&mut self) -> usize {
        //nice
        let body_count : usize= self.content
            .iter()
            .map(|message| {
                message.get_body_lines(self.size.0)
            })
            .sum();

         let max = std::cmp::max(
            //account for lines and widget height
            (body_count + 2*self.content.len()) as i32 - 28, 0
        ) as usize; 

        self.max_offset = max;
        max
    }

    pub fn scroll_to_bottom(&mut self) {
        let scroll = self.get_max_offset(); //widget height
        if scroll > 0{
            self.scroll = scroll as usize;
            return;
        }
        self.scroll = 0
    }
    pub fn input_mode(&self) -> InputMode{ 
        return self.input_mode;
    }

    pub fn set_input_mode(&mut self, mode: InputMode) {
        match mode {
            InputMode::Command => {
                self.command_status = CommandStatus::Okay;
                self.command = String::from(':');
            }
            _ => {}
        }
        self.input_mode = mode;
    }

    pub fn scroll_down (&mut self) { 
        if self.scroll != 0 {
            self.scroll -= 1;
        }
    }

    pub fn get_scroll(&self) -> usize {
        return self.scroll;
    }

    pub fn scroll_up (&mut self) { 
        if self.scroll < self.max_offset {
            self.scroll += 1;
        }
        
    }
    
    pub fn get_display_input(&self) -> String {
        return self.display_input.clone()
    }

    pub fn get_input(&self) -> String{
        return self.internal_input.clone();
    }

    pub fn push_input(&mut self, char: char) {
        self.display_input.push(char);
    }
    
    pub fn pop_input(&mut self) {
        self.display_input.pop();
    }

    pub fn push_answer(&mut self,message: DisplayMessage) {
        self.content.push(message);
        self.internal_input = String::new();
   }


    pub fn push_content(&mut self,
        sender: String,
        message_type: MessageType,
        message: String,
    ) {
        let message = DisplayMessage {
            sender,
            body: message,
            message_type,
        };

        self.content.push(message);
   }



    pub fn get_content(&self) -> Vec<Spans>{

        let mut span_vec: Vec<Spans> = Vec::new();
        for message in &self.content{
            let sender_spans = self.sender_from(&message);
            let body_spans = self.body_from(&message);

            let line = self.line_from(&message.message_type);

            span_vec.push(line);
            span_vec.push(sender_spans);
            for spans in body_spans {
                span_vec.push(spans);
            }
        }



        return span_vec;
    }

    pub fn update_size(&mut self) {
        self.size = get_terminal_sizes();
    }

    fn body_from <'a> (&self, message: &'a DisplayMessage) -> Vec<Spans<'a>> {

        match message.message_type {
            MessageType::Query => {
                vec![
                    Spans::from(vec![Span::raw(message.body.clone())])
                ]
            }
            MessageType::Answer => {
                self.parse_answer(&message.body)
            }
        }
    }
    fn sender_from<'a>(&self, message: &'a DisplayMessage) -> Spans<'a> {

        match message.message_type {

            MessageType::Query => {
                Spans::from(vec![
                    Span::styled(
                        &message.sender,
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Blue)
                    ),
                    Span::raw(":"),
                ])
            }
            MessageType::Answer => {
                Spans::from(vec![
                    Span::styled(
                        &message.sender,
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Magenta)
                    ),
                    Span::raw(":"),
                ])
            }
        }
    }

    fn line_from<'a>(&self, message_type:  &MessageType) -> Spans<'a>{
        match message_type {
            MessageType::Query => {
                Spans::from (
                    vec![Span::styled(format!("{:─>width$}","",width=self.size.0 as usize ),
                        Style::default().fg(Color::Blue))]
                )
            }
            MessageType::Answer => {
                Spans::from (
                    vec![Span::styled(format!("{:─>width$}","",width=self.size.0 as usize ),
                        Style::default().fg(Color::Magenta))]
                )
            }
        }
    }

    fn parse_answer<'a>(&self, answer: &'a str) -> Vec<Spans<'a>> {
        let partial = answer.split('\n').collect::<Vec<&str>>();

        let mut span_vec: Vec<Spans> = Vec::new();

        for line in partial {
            span_vec.push(
                Spans::from(vec![Span::raw(line)])
            );
        }

        return span_vec;
    }
    pub fn set_username(&mut self, name: String) {
        self.username = name;
    }
    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_command(&self) -> String {
        return self.command.clone()
    }

    pub fn send_command(&mut self) {
        let commands: Vec<String> = Vec::new();

        if commands.contains(&self.command) {
            // self.set_input_mode(InputMode::Normal);
        } else { 
            self.command = "Error: Command not found".to_string();
            self.set_input_mode(InputMode::Normal);
            self.command_status = CommandStatus::Error;
        }
        
    }

    pub fn push_command(&mut self, c: char) {
        self.command.push(c)
    }

    pub fn pop_command(&mut self){
        self.command.pop();
    }

    pub fn reset_command(&mut self){
        self.command = String::from("");
        self.command_status = CommandStatus::Okay;
    }

    pub fn command_status(&self) -> &CommandStatus {
        return &self.command_status
    }
}


impl Default for App {
    fn default() -> App {
        App {
            should_end: false,
            scroll: 0,

            username: String::new(),

            display_input: String::new(),
            internal_input: String::new(),
            input_mode: InputMode::Normal, 

            content: Vec::new(),

            command: String::from(""),
            command_status: CommandStatus::Okay,

            size: get_terminal_sizes(),
            max_offset: 0,

            api_manager: None,

        }
    }
}


