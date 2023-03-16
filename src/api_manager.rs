
use crate::app::DisplayMessage;
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


    pub fn new(token: String) -> ApiManager {
        let mut chat_api = ChatApi::default();
        chat_api.load_file("messages.json".to_string());
        return ApiManager {
            text_api_handler: TextApi::new(),
            chat_api_handler: chat_api,
            token
        }
    }


    pub fn load_chat(&mut self, filename: String) {
        self.chat_api_handler.load_file(String::from(filename.trim()));
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
