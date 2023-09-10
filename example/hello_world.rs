use edge_tts_rs::edge_api::{EdgeTTS, EdgeTTSConfig, TTS};
use std::error::Error;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tts = EdgeTTS::new(EdgeTTSConfig::default());
    let mut client = tts.connect()?;
    let vec = tts
        .send_content(&mut client, "Hello, World".to_string())
        .await?;
    fs::write("hello_world.mp3", vec).unwrap();

    Ok(())
}
