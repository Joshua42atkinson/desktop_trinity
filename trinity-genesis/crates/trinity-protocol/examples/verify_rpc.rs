use tarpc::{client, context, tokio_serde::formats::Bincode};
use tokio_util::codec::LengthDelimitedCodec;
use trinity_protocol::{brain::BrainServiceClient, ChatMessage};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_addr: SocketAddr = "127.0.0.1:9000".parse()?;
    let stream = tokio::net::TcpStream::connect(server_addr).await?;
    
    let codec = LengthDelimitedCodec::new();
    let framed = tokio_util::codec::Framed::new(stream, codec);
    let transport = tarpc::serde_transport::new(framed, Bincode::default());
    
    let client = BrainServiceClient::new(client::Config::default(), transport).spawn();
    
    println!("🧪 Calling Trinity Brain API (chat)...");
    
    let message = ChatMessage {
        role: "user".to_string(),
        content: "Who are you?".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    let response = client.chat(context::current(), message, vec![]).await?;
    
    println!("✅ Trinity responded: \n\n{}", response);
    
    Ok(())
}
