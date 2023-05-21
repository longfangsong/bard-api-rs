use rand::Rng;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatSession {
    request_id: i32,
    snlm0e: String,
    cfb2h: String,
    last_conversation_id: String,
    last_response_id: String,
    last_choice_id: String,
}

impl ChatSession {
    pub async fn new() -> Self {
        let mut client = reqwest::Client::builder();
        if let Ok(proxy) = std::env::var("https_proxy") {
            client = client.proxy(reqwest::Proxy::https(&proxy).unwrap());
        }
        let mut headers = header::HeaderMap::new();
        let secure_1psid = std::env::var("Secure_1PSID").unwrap();
        let secure_1psid = format!("__Secure-1PSID={secure_1psid}");
        headers.insert(
            "Cookie",
            header::HeaderValue::from_str(&secure_1psid).unwrap(),
        );
        let response = client
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36")
            .default_headers(headers)
            .build()
            .unwrap()
            .get("https://bard.google.com/").send().await.unwrap();
        let response_text = response.text().await.unwrap();
        let snlm0e_search_result = regex::Regex::new(r#"SNlM0e":"(.*?)""#)
            .unwrap()
            .captures(&response_text);
        let cfb2h_search_result = regex::Regex::new(r#"cfb2h":"(.*?)""#)
            .unwrap()
            .captures(&response_text);
        if snlm0e_search_result.is_none() || cfb2h_search_result.is_none() {
            panic!("Cannot find snlm0e or cfb2h");
        } else {
            let snlm0e = snlm0e_search_result
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .to_string();
            let cfb2h = cfb2h_search_result
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .to_string();
            let request_id = rand::thread_rng().gen_range(100000..999999);
            ChatSession {
                request_id,
                snlm0e,
                cfb2h,
                last_conversation_id: "".to_string(),
                last_response_id: "".to_string(),
                last_choice_id: "".to_string(),
            }
        }
    }

    fn parse_response(&self, response: &str) -> (String, String, String, String) {
        let mut lines = response.split('\n');
        let the_line = lines.find(|it| it.contains("wrb.fr")).unwrap();
        let parsed_line = serde_json::from_str::<Vec<Vec<serde_json::Value>>>(the_line).unwrap();
        let inner_str = parsed_line[0][2].as_str().unwrap();
        let inner = serde_json::from_str::<Vec<Vec<serde_json::Value>>>(&inner_str).unwrap();
        let text_response = inner[0][0].as_str().unwrap();
        let c_and_r = [inner[1][0].as_str().unwrap(), inner[1][1].as_str().unwrap()];
        let rc = inner[4][0].as_array().unwrap()[0].as_str().unwrap();
        return (
            text_response.to_string(),
            c_and_r[0].to_string(),
            c_and_r[1].to_string(),
            rc.to_string(),
        );
    }

    pub async fn send_message(&mut self, text: &str) -> String {
        let input_text_struct = json!([
            [text.to_string()],
            null,
            [
                self.last_conversation_id.clone(),
                self.last_response_id.clone(),
                self.last_choice_id.clone(),
            ],
        ]);
        let input_text = serde_json::to_string(&input_text_struct).unwrap();
        let mut params = Vec::new();
        params.push(("bl", self.cfb2h.clone()));
        params.push(("_reqid", self.request_id.to_string()));
        params.push(("rt", "c".to_string()));
        params.push((
            "f.req",
            serde_json::to_string(&json!([null, input_text])).unwrap(),
        ));
        params.push(("at", self.snlm0e.clone()));
        let mut client = reqwest::Client::builder();
        if let Ok(proxy) = std::env::var("https_proxy") {
            client = client.proxy(reqwest::Proxy::https(&proxy).unwrap());
        }
        let mut headers = header::HeaderMap::new();
        let secure_1psid = std::env::var("Secure_1PSID").unwrap();
        let secure_1psid = format!("__Secure-1PSID={secure_1psid}");
        headers.insert(
            "Cookie",
            header::HeaderValue::from_str(&secure_1psid).unwrap(),
        );
        let resp = client
            .default_headers(headers)
            .build()
            .unwrap()
            .post("https://bard.google.com/_/BardChatUi/data/assistant.lamda.BardFrontendService/StreamGenerate")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .unwrap();
        let resp_text = resp.text().await.unwrap();
        let (text_response, conversation_id, response_id, choice_id) =
            self.parse_response(&resp_text);
        self.last_conversation_id = conversation_id;
        self.last_response_id = response_id;
        self.last_choice_id = choice_id;
        self.request_id += 100000;
        text_response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let mut session = ChatSession::new().await;
        let response = session.send_message("Hello").await;
        println!("{}", response);
    }
}
