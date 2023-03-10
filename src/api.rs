//logging
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};
use std::future::Future;

use serde::{Deserialize,Serialize};

use crate::app::{Message, MessageType};


#[derive(Deserialize, Debug)]
pub struct Choices {
    text: String,
    // index: i32,
    // logprobs: Option<String>,
    // finish_reason: String
}
#[derive(Deserialize, Debug)]
pub struct Usage {
    // prompt_tokens : i32,
    // completion_tokens: i32,
    // total_tokens: i32
}
#[derive(Deserialize,Debug)]
pub struct ApiResponse {
    // id: String,
    // object: String,
    // created: i32,
    model: String,
    choices: Vec<Choices>,
    // usage: Usage,
}


#[derive(Serialize,Deserialize,Debug)]
pub struct ApiCall {
    model: String,
    prompt: String,
    temperature: u8,
    max_tokens: i32,
}

impl ApiCall {
    fn from(
        model: String,
        prompt: String,
        temperature: u8,
        max_tokens: i32
    ) -> ApiCall {
        return ApiCall {
            model,
            prompt,
            temperature,
            max_tokens
        }
        
    }

    fn get_prompt(&self) -> String {
        return self.prompt.clone()
    }
    fn get_model(&self) -> String {
        return self.model.clone();
    }
}

pub struct ApiHandler {
    client: reqwest::Client,
    token: String,
    pub call: Option<ApiCall>,
    pub response: Option<ApiResponse>
}
impl ApiHandler {
    pub fn new(token:String) -> ApiHandler {
        return ApiHandler {
            client: reqwest::Client::new(),
            token,
            call: None,
            response: None
        }
    }


    pub(crate) fn update_call(&mut self,call: ApiCall) {
        self.call = Some(call);
    }

    async fn send_dummy_api_reqwest(
        &self,
        call_params: &ApiCall,

    ) -> Result<ApiResponse, reqwest::Error> {
        let res = self.client.get(
            format!("http://127.0.0.1:7878/{}",call_params.get_prompt())
        )
            .send()
            .await;
        
        match res {
            Ok(okay) => {
                let debug_json = format!("{:#?}",okay);
                let res = okay
                    .json::<ApiResponse>();

                let json_res = res.await;

                match json_res {
                    Ok(okay) => return Ok(okay),
                    Err(err) => {
                        error!("Couldn't parse received request");
                        debug!("RECEIVED: {}",debug_json);
                        return Err(err)
                    }
                    
                }
            }
            Err(err) => {
                error!("Couldn't get api response");
                return Err(err)
            }
        }

    }
    //this function works, that's very nice
    async fn send_api_reqwest(
        &self,
        call_params: &ApiCall,

    ) -> Result<ApiResponse, reqwest::Error> {


        let call = ApiCall {
            model: "text-davinci-003".to_string(),
            prompt: "Say this is a test".to_string(),
            temperature: 0,
            max_tokens: 7    
        };
        // curl https://api.openai.com/v1/completions \
        // -H "Content-Type: application/json" \
        // -H "Authorization: Bearer YOUR_API_KEY" \
        // -d '{"model": "text-davinci-003", "prompt": "Say this is a test", "temperature": 0, "max_tokens": 7}'
        let res = self.client.post(
            "https://api.openai.com/v1/completions"
        )
        .bearer_auth(&self.token)
        .header("Content-Type", "application/json")
        .json(call_params)
        .send()
        .await;

        match res {
            Ok(okay) => {
                let debug_json = format!("{:#?}",okay);
                let res = okay
                    .json::<ApiResponse>();

                let json_res = res.await;

                match json_res {
                    Ok(res) => return Ok(res),
                    Err(err) => {
                        error!("Couldn't parse received request");
                        debug!("RECEIVED: {}",debug_json);
                        return Err(err)
                    }
                    
                }
            }
            Err(err) => {
                error!("Couldn't get api response");
                return Err(err)
            }
        }
    }



    pub async fn answer_from(
        &mut self,
        model: String,
        query: String,
        temperature: u8,
        max_tokens: i32
        ) -> crate::app::Message {

        self.update_call(ApiCall::from(model,query,temperature,max_tokens));
        let call = match &self.call  {
            Some(call) => call,
            None => {
                panic!("Should be able to build apicall")
            }
        };
        let answer = self.send_dummy_api_reqwest(
        // let answer = self.send_api_reqwest(
            &call
        ).await;

        self.response = match answer {
            Ok(ans) => {
                Some(ans)
            }
            Err(_) => {
                error!("Couldn't get answer");
                error!("ERROR: {:#?}", answer);
                None
            }
        };

        self.message_from_answer(self.response.as_ref())
    }

    fn message_from_answer(&self, answer: Option<&ApiResponse>) -> Message {

        let sender: String;
        let body: String;
        match answer {
            Some(answer) => {
                sender = answer.get_model();
                body = answer.choices()[0].get_answer();
            }
            None => {
                sender = "YAS - your average system".to_string();
                body = "something went wrong in the request (probably problem
                parsing the response), try again".to_string();
            }
        }

        return Message::from(sender,body,MessageType::Answer) 
    }
}


impl ApiResponse {
    pub fn choices(&self) -> &Vec<Choices> {
        return &self.choices;
    }
    pub fn get_model(&self) -> String {
        return String::from(&self.model);
    }
}


impl Choices {
    pub fn get_answer(&self) -> String {
        return String::from(&self.text);
    }
}



