
use crate::app::{AppError,DisplayMessage};
use crate::text_api::{TextApi};
use crate::chat_api::{ChatApi};

pub enum CallType {
    Chat,
    Text
}


pub struct ApiManager {
    text_api_handler: TextApi,
    chat_api_handler: ChatApi,
    token: String,
}

impl ApiManager {


    pub fn new(token: String) -> Result<ApiManager,AppError> {
        let mut chat_api = ChatApi::default();
        match chat_api.load_file("messages.json".to_string()) {
            Ok(_) => {
                return Ok(ApiManager {
                    text_api_handler: TextApi::new(),
                    chat_api_handler: chat_api,
                    token
                })
            }
            Err(err) => {
                return Err(err)
            }
        }
    }


    pub fn load_chat(&mut self, filename: String) -> Result< (), AppError> {
        self.chat_api_handler.load_file(String::from(filename.trim()))?;
        Ok(())
    }
    pub fn get_display_chat(&self) -> Vec<DisplayMessage> {
        self.chat_api_handler.get_display_chat()
    }
    pub async fn answer_from (
        &mut self,
        query: String,
        call_type: CallType
    ) -> crate::app::DisplayMessage {
        match call_type {
            CallType::Text => {
                self.text_api_handler.answer_from(query,self.token.clone()).await
            }
            CallType::Chat => {
                self.chat_api_handler.answer_from(query,self.token.clone()).await
            }
        }
    }
}
