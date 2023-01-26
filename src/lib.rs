use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use json::JsonValue;
use std::error::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

const WS_DEBUGGER_URL: &str = "webSocketDebuggerUrl";

pub mod cdp_messages;

#[derive()]
pub struct ChromiumBrowser
{
    connection_url: String,
    write_sink: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    read_stream: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}

impl ChromiumBrowser
{
    pub fn new(connection_url: String) -> Self
    {
        return ChromiumBrowser {
            connection_url,
            write_sink: None,
            read_stream: None,
        };
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>>
    {
        let response = reqwest::get(&self.connection_url)
            .await
            .unwrap()
            .text()
            .await
            .expect("Cannot get response from /json");

        let response_json = json::parse(&response).expect("Cannot parse response into json");

        let session_url = response_json[0][WS_DEBUGGER_URL]
            .as_str()
            .expect("Cannot get debugger url");

        let chrome_url = url::Url::parse(session_url).unwrap();

        let (ws_stream, _) = connect_async(chrome_url).await.expect("Failed to connect");

        let (write, read) = ws_stream.split();
        self.write_sink = Some(write);
        self.read_stream = Some(read);
        println!("WebSocket handshake has been successfully completed");

        Ok(())
    }

    pub async fn run_command(
        &mut self,
        command: &mut JsonValue,
    ) -> Result<(), Box<dyn Error>>
    {
        command["id"] = 1.into();

        match self
            .write_sink
            .as_mut()
            .unwrap()
            .send(Message::text(command.dump()))
            .await
        {
            Ok(()) => println!("Command end to chrome"),
            Err(error) => println!("Command errored: {error}"),
        };

        Ok(())
    }
}
