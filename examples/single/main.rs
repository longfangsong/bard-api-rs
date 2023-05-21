use bard_api_rs::ChatSession;
use ezio::prelude::*;

#[tokio::main]
async fn main() {
    let mut session = ChatSession::new().await;
    println!("Ask the question please:");
    let question = stdio::read_line();
    let response = session.send_message(&question).await;
    println!(">> {}", response);
}
