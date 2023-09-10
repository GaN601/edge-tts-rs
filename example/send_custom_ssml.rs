use edge_tts_rs::edge_api::{EdgeTTS, EdgeTTSConfig, TTS};
use std::error::Error;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tts = EdgeTTS::new(EdgeTTSConfig::default());
    let mut client = tts.connect()?;
    let vec = tts.send_ssml(&mut client,
            r#"<speak xmlns="http://www.w3.org/2001/10/synthesis" xmlns:mstts="http://www.w3.org/2001/mstts" xmlns:emo="http://www.w3.org/2009/10/emotionml" version="1.0" xml:lang="en-US">
        <voice name="zh-CN-XiaoxiaoNeural">
                <prosody rate="0%" pitch="0%">
                你好啊, 今天天气怎么样?
                </prosody>
        </voice>
    </speak>"#.to_string(), )
        .await?;
    fs::write("send_custom_ssml.mp3", vec).unwrap();

    Ok(())
}
