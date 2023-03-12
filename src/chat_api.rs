//logging
use crate::app::{DisplayMessage, MessageType};
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};
use serde::{Deserialize, Serialize};
use std::{future::Future, fs::File, io::{BufReader, Read}};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ChatApiRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

impl ChatApiRequest {
    pub fn from(model: String, messages: Vec<ChatMessage>) -> Self {
        ChatApiRequest{model, messages}
    }
}

#[derive(Deserialize, Debug)]
struct ChatApiResponse {
    id: String,
    object: String,
    created: i64,
    // choices: Vec<ChatChoice>,
    // usage: Usage,
}

// #[derive(Deserailize,Debug)]
struct ChatChoice {
    index: i32,
    message: ChatMessage,
    finish_reason: String,
}
pub(crate) struct ChatApi {
    client: reqwest::Client,
    request: Option<ChatApiRequest>,
    response: Option<ChatApiResponse>,
    selected_model: String,
    chat: Vec<ChatMessage>
}

impl ChatApi {
    pub fn new() -> ChatApi {
        ChatApi {
            client: reqwest::Client::new(),
            request: None,
            response: None,
            selected_model: String::new(),
            chat: vec![],
        }
    }
    pub fn load_file(&mut self, filename: String) {

        let file = File::open(filename).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();

        let messages: Vec<ChatMessage> = serde_json::from_str(contents.as_str()).expect("couldn't parse chat");
        self.chat = messages;
        debug!("{:#?}", self.chat);
    }
    pub fn send_display_chat(&self) -> Vec<DisplayMessage> {
        let mut display_chat = vec![];
        for message in &self.chat {
            display_chat.push(self.translate_to_display(message));
        }
        return display_chat
    }
    pub fn translate_to_display(&self, message: &ChatMessage) -> DisplayMessage {
        if message.role == "user" {
            DisplayMessage::from("user".to_string(), message.content.clone(), MessageType::Query)
        }
        else {
            DisplayMessage::from("some_model".to_string(), message.content.clone(), MessageType::Query)
        }
    }
    fn update_call(&mut self, request: ChatApiRequest){
        self.request = Some(request);
    }
    //
    pub fn answer_from(&mut self, query: String, token: String) -> DisplayMessage {
        DisplayMessage::from( "me".to_string(),"body".to_string(), MessageType::Answer)
    }
}

impl Default for ChatApi {
    fn default() -> ChatApi {
        ChatApi {
            client: reqwest::Client::new(),
            request: None,
            response: None,
            selected_model: String::new(),
            chat: vec![],
        }
    }
}
