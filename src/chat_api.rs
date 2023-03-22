//logging
use crate::app::{DisplayMessage, MessageType, AppError};
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    future::Future,
    io::{BufReader, Read},
};

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct ChatMessage {
    role: String,
    content: String,
}

impl ChatMessage {
    pub fn from(role: String, content: String) -> ChatMessage {
        ChatMessage { role, content }
    }
}


#[derive(Serialize, Deserialize, Debug)]
struct ChatApiRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

impl ChatApiRequest {
    pub fn from(model: String, messages: Vec<ChatMessage>) -> Self {
        ChatApiRequest { model, messages }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct ChatApiResponse {
    id: String,
    object: String,
    created: i64,
    choices: Vec<ChatChoice>,
    // usage: Usage,
}

impl ChatApiResponse {
    pub fn get_first_choice(&self) -> ChatChoice {
        self.choices[0].clone()
    }
}

#[derive(Serialize, Clone, Deserialize, Debug)]
struct ChatChoice {
    index: i32,
    message: ChatMessage,
    finish_reason: String,
}

impl ChatChoice {
    pub fn get_message(&self) -> ChatMessage {
        self.message.clone()
    }
}
pub(crate) struct ChatApi {
    client: reqwest::Client,
    request: Option<ChatApiRequest>,
    response: Option<ChatApiResponse>,
    selected_model: String,
    chat: Vec<ChatMessage>,
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
    pub fn load_file(&mut self, filename: String) -> Result <(), AppError>{
        
        let file = File::open(filename.clone())?;
            // .expect(format!("should be able to open file {} passed none??", filename.clone()).as_str());
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();


        let messages: Vec<ChatMessage> = serde_json::from_str(contents.as_str())?;

        //let messages: Vec<ChatMessage> = serde_json::from_str(contents.as_str()).expect("couldn't parse chat");

        // match serde_json::from_str(contents.as_str())
        //  {
        //     Ok(parsed_messages) => {
        //         let messages: Vec<ChatMessage> = parsed_messages;
        //         self.chat = messages;
        //     }
        //     Err(_) =>{
        //         Err(return AppError {
        //             message: "Couldn't parse chat file".to_string();
        //         })
        //     }
        // }
        // let messages: Vec<ChatMessage> =
        // 

        self.chat = messages;
        debug!("Chat is:{:#?}", self.chat);
        Ok(())
    }

    pub fn get_display_chat(&self) -> Vec<DisplayMessage> {
        let mut display_chat :Vec<DisplayMessage> = vec![];
        for message in &self.chat{
            display_chat.push(self.translate_to_display(&message));
        }
        display_chat
            .into_iter()
            .filter(|x| x.get_sender() != "system")
            .collect()
    }
    //
    pub fn translate_to_display(&self, message: &ChatMessage) -> DisplayMessage {
        if message.role == "user" {
            DisplayMessage::new(
                "user".to_string(),
                message.content.clone(),
                MessageType::Query,
            )
        } 
        
        else if message.role == "system" {
            DisplayMessage::new(
                "system".to_string(),
                "".to_string(),
                MessageType::Query,
            )
        } 
        else {
            DisplayMessage::new(
                self.selected_model.clone(),
                message.content.clone(),
                MessageType::Answer,
            )
        }
    }



    fn update_query(&mut self, query: String) {
        let query = ChatMessage::from(String::from("user"), query);
        self.chat.push(query)
    }
    fn update_request(&mut self) {
        self.request = Some(ChatApiRequest::from(String::from("gpt-3.5-turbo"),
            self.chat.clone()))
    }
    //

    async fn send_api_reqwest(
        &mut self,
        token: String,
    ) -> Result<ChatApiResponse, reqwest::Error> {

        // -H 'Content-Type: application/json' \
        //       curl https://api.openai.com/v1/chat/completions \
        // -H 'Authorization: Bearer YOUR_API_KEY' \
        // -d '{
        // "model": "gpt-3.5-turbo",
        // "messages": [{"role": "user", "content": "Hello!"}]


        self.update_request();

        let res = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .json(&self.request)
            .send()
            .await;

        debug!("{:#?}", self.request);


        match res {
            Ok(okay) => {
                let debug_json = format!("{:#?}", okay);
                let res = okay.json::<ChatApiResponse>();

                let json_res = res.await;

                match json_res {
                    Ok(res) => {
                        debug!("CHAT IS: {:#?}", self.chat.clone());
                        return Ok(res);
                    }
                    Err(error) => {
                        debug!("CHAT IS: {:#?}", self.chat.clone());
                        error!("Couldn't parse the received request");
                        debug!("Received: {}", debug_json);
                        return Err(error);
                    }
                }
            }
            Err(error) => {
                error!("Couldn't get api response");
                return Err(error);
            }
        }
    }

    fn update_chat(&mut self, response: ChatApiResponse) {
        let message = response.get_first_choice().get_message();
        self.chat.push(message)
    }

    pub async fn answer_from(&mut self, query: String, token: String) -> DisplayMessage {

        self.update_query(query.clone());
        let res = self.send_api_reqwest(token).await;

        debug!("{:#?}", res);
        match res {
            Ok(res) => {
                self.update_chat(res.clone());
                let mes = self.translate_to_display(&res.get_first_choice().get_message());

                debug!("{:#?}", mes.clone());
                return mes

            }
            Err(_) => {
                error!("Couldn't get api response");
                DisplayMessage::error(String::from("Some error occurred fetching the api, please
                    try again"))
            }


        }
    }
}

impl Default for ChatApi {
    fn default() -> ChatApi {
        ChatApi {
            client: reqwest::Client::new(),
            request: None,
            response: None,
            selected_model: String::from("Gpt"),
            chat: vec![],
        }
    }
}
