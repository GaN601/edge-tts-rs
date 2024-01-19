mod reg_log;

use crate::reg_log::reg_log;
use edge_tts_rs::edge_api::{EdgeTTS, EdgeTTSConfig, TTS};
use std::error::Error;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    reg_log()?;

    let tts = EdgeTTS::new(EdgeTTSConfig::default());
    let mut client = tts.connect()?;
    let mut vec = tts.send_content(&mut client, "Hello".to_string()).await?;
    let mut vec1 = tts.send_content(&mut client, "World".to_string()).await?;
    vec.append(&mut vec1);
    fs::write("hello_world.mp3", vec).unwrap();

    Ok(())
}
