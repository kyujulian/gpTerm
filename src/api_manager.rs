
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
