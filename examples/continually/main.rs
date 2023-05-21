use std::{fs::File, path::PathBuf};

use bard_api_rs::ChatSession;
use clap::Parser;
use ezio::prelude::*;

/// Chat with new Bing continually
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Create a new chat session with bing, and store the dumped session somewhere.
    #[arg(long, group = "input")]
    create: Option<PathBuf>,

    /// Load a dumped session and and continue the chat.
    #[arg(long, group = "input")]
    load: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Some(target_path) = args.create {
        let mut session = ChatSession::new().await;
        println!("Ask the question please:");
        let question = stdio::read_line();
        println!("Waiting for bing for response ...");
        let response = session.send_message(&question).await;
        println!(">> {}", response);
        let file = File::create(&target_path).unwrap();
        serde_json::to_writer(file, &session).unwrap();
    } else if let Some(source_path) = args.load {
        let file = File::open(&source_path).unwrap();
        let mut session: ChatSession = serde_json::from_reader(file).unwrap();
        println!("Ask the question please:");
        let question = stdio::read_line();
        println!("Waiting for bing for response ...");
        let response = session.send_message(&question).await;
        println!(">> {}", response);
        let file = File::create(&source_path).unwrap();
        serde_json::to_writer(file, &session).unwrap();
    }
}
