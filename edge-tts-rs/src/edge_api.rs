use std::error::Error;

use std::str::FromStr;
use websocket::client::ParseError;
use websocket::header::{Headers, UserAgent};
use websocket::OwnedMessage::Text;
use websocket::{Message, OwnedMessage};

use crate::util::{gen_request_id, now_millis};
use websocket::stream::sync::NetworkStream;
use websocket::sync::Client;
use websocket::url::Url;

type TTSSocket = Client<Box<dyn NetworkStream + Send>>;
pub trait TTS {
    /// connect TTS service
    fn connect(&self) -> Result<TTSSocket, Box<dyn Error>>;
}

pub struct EdgeTTS {
    pub(crate) config: EdgeTTSConfig,
    request_id: String,
    headers: Headers,
}

#[derive(Debug)]
pub struct EdgeTTSConfig {
    /// refer to SuggestedCodec in https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken=6A5AA1D4EAFF4E9FB37E23D68491D6F4
    // todo check the output format
    pub output_format: String,
    pub voice_name: String,
    /// refer to rate in https://learn.microsoft.com/zh-cn/azure/ai-services/speech-service/speech-synthesis-markup-voice#adjust-prosody
    pub rate: f32,
    pub pitch: f32,
}

impl Default for EdgeTTSConfig {
    fn default() -> Self {
        Self {
            output_format: String::from("audio-24khz-96kbitrate-mono-mp3"),
            voice_name: String::from("zh-CN-XiaoxiaoNeural"),
            rate: 0.0,
            pitch: 0.0,
        }
    }
}

impl EdgeTTSConfig {
    fn to_config_message(&self) -> String {
        let json_first = r#"{"context": {"synthesis": {"audio": {"metadataoptions": {"sentenceBoundaryEnabled": "false","wordBoundaryEnabled": "false"},"outputFormat": "#;
        let json_last = r#"}}}}"#;
        let json = format!(r#"{}"{}"{}"#, json_first, self.output_format, json_last);
        let string = format!(
            "X-Timestamp:{}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{}",
            now_millis(), json
        );
        println!("speech config {}", string);
        string
    }
    pub(crate) fn to_ssml(&self, content: String) -> String {
        format!(
            r#"<speak xmlns="http://www.w3.org/2001/10/synthesis" xmlns:mstts="http://www.w3.org/2001/mstts" xmlns:emo="http://www.w3.org/2009/10/emotionml" version="1.0" xml:lang="en-US"><voice name="{}"><prosody rate="{}" pitch="{}">{}</prosody ></voice > </speak >"#,
            self.voice_name, self.rate, self.pitch, content
        )
    }
}

impl EdgeTTS {
    pub fn new(config: EdgeTTSConfig) -> Self {
        let mut headers = Headers::new();
        headers.set(UserAgent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.5060.66 Safari/537.36 Edg/103.0.1264.44".to_owned()));
        Self {
            config,
            request_id: gen_request_id(),
            headers,
        }
    }
    pub(crate) fn connect_url(&self) -> Result<Url, ParseError> {
        Url::from_str(
        format!("wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken={}&ConnectionId={}",
                "6A5AA1D4EAFF4E9FB37E23D68491D6F4",
                self.request_id).as_str())
    }

    /// send text to TTS simple method wrapper
    ///
    /// # Arguments
    ///
    /// * `client`: connected EdgeTTS WebSocket
    /// * `content`: send text
    ///
    /// returns: Result<Vec<u8, Global>, Box<dyn Error, Global>>
    ///
    /// # Examples
    ///
    /// ```
    /// use std::error::Error;
    /// use std::future::Future;
    /// use edge_tts_rs::edge_api::{EdgeTTS, EdgeTTSConfig, TTS};
    ///
    ///
    /// let tts = EdgeTTS::new(EdgeTTSConfig::default());
    /// let mut client = tts.connect().unwrap();
    /// // async {  tts.send(&mut client,"Hello").await};
    /// ```
    pub async fn send_content(
        &self,
        client: &mut TTSSocket,
        content: String,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let ssml = self.config.to_ssml(content);
        self.send_ssml(client, ssml).await
    }

    /// send ssml to TTS simple method wrapper
    ///
    /// # Arguments
    ///
    /// * `client`: connected EdgeTTS WebSocket
    /// * `content`: send text
    ///
    /// returns: Result<Vec<u8, Global>, Box<dyn Error, Global>>
    ///
    /// # Examples
    ///
    /// ```
    /// use std::error::Error;
    /// use std::future::Future;
    /// use edge_tts_rs::edge_api::{EdgeTTS, EdgeTTSConfig, TTS};
    ///
    /// let ssml = String::from("");
    /// let tts = EdgeTTS::new(EdgeTTSConfig::default());
    /// let mut client = tts.connect().unwrap();
    /// // async {  tts.send(&mut client,ssml).await};
    /// ```
    pub async fn send_ssml(
        &self,
        client: &mut TTSSocket,
        ssml: String,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        println!("ssml prepared: {}", ssml);

        let message = Message::text(format!("X-Timestamp:{}\r\nX-RequestId:{}\r\nContent-Type:application/ssml+xml\r\nPath:ssml\r\n\r\n{}",now_millis(),self.request_id,ssml));
        client.send_message(&message)?;

        let mut voice_binary: Vec<u8> = Vec::with_capacity(1024);
        let mut flag: bool = false;
        'l: loop {
            let resp = client.recv_message()?;
            match resp {
                Text(resp) => {
                    println!("{}", resp);
                    // todo receive example:
                    // keynote: turn.start 1. Text("X-RequestId:ef0e87998a3d4115a0be2e637f5aaed8\r\nContent-Type:application/json; charset=utf-8\r\nPath:turn.start\r\n\r\n{\n  \"context\": {\n    \"serviceTag\": \"57be03b6dae64e2ab82972e81f796ba0\"\n  }\n}")
                    // 2. Text("X-RequestId:ef0e87998a3d4115a0be2e637f5aaed8\r\nContent-Type:application/json; charset=utf-8\r\nPath:response\r\n\r\n{\"context\":{\"serviceTag\":\"57be03b6dae64e2ab82972e81f796ba0\"},\"audio\":{\"type\":\"inline\",\"streamId\":\"0B38A07F8AE1437AB16D46C71CF3ECBB\"}}")
                    // voice info:  3. Text("X-RequestId:ef0e87998a3d4115a0be2e637f5aaed8\r\nContent-Type:application/json; charset=utf-8\r\nPath:audio.metadata\r\n\r\n{\n  \"Metadata\": [\n    {\n      \"Type\": \"WordBoundary\",\n      \"Data\": {\n        \"Offset\": 1000000,\n        \"Duration\": 5625000,\n        \"text\": {\n          \"Text\": \"Hello\",\n          \"Length\": 5,\n          \"BoundaryType\": \"WordBoundary\"\n        }\n      }\n    }\n  ]\n}")
                    // keynote: turn.end 4. message response: Text("X-RequestId:ef0e87998a3d4115a0be2e637f5aaed8\r\nContent-Type:application/json; charset=utf-8\r\nPath:turn.end\r\n\r\n{}")
                    if resp.contains("turn.start") {
                        flag = true;
                    } else if resp.contains("turn.end") {
                        break 'l;
                    }
                }
                OwnedMessage::Binary(mut resp) => {
                    if flag {
                        voice_binary.append(&mut resp);
                    }
                }
                OwnedMessage::Close(resp) => {
                    println!("{:?}", resp);
                    return match resp {
                        None => Err("the socket closed".to_string().into()),
                        Some(reason) => Err(reason.reason.into()),
                    };
                }
                _ => break 'l,
            }
        }
        Ok(voice_binary)
    }
}
impl TTS for EdgeTTS {
    fn connect(&self) -> Result<TTSSocket, Box<dyn Error>> {
        let url = self.connect_url()?;
        let mut builder = websocket::ClientBuilder::from_url(&url)
            .custom_headers(&self.headers)
            .origin("chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold".to_string());

        match builder.connect(None) {
            Ok(mut c) => {
                let message = Message::text(self.config.to_config_message());
                c.send_message(&message)?;
                Ok(c)
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}
