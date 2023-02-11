use log::trace;
use serde::Deserialize;
use serde_json::Value;
use std::{error::Error, net::TcpStream};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

#[derive()]
pub struct ChromiumBrowser
{
    websocket: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl ChromiumBrowser
{
    pub fn connect_with_client(
        client: &dyn ChromeAPI,
        url: &String,
    ) -> Result<ChromiumBrowser, Box<dyn Error>>
    {
        let web_socket = ChromiumBrowser::get_websocket(client, &url)?;
        trace!("Create websocket success");
        Ok(ChromiumBrowser {
            websocket: web_socket,
        })
    }

    pub fn connect(url: &String) -> Result<ChromiumBrowser, Box<dyn Error>>
    {
        let client = ChromeAPIClient {};

        ChromiumBrowser::connect_with_client(&client, url)
    }

    fn get_websocket(
        client: &dyn ChromeAPI,
        url: &String,
    ) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Box<dyn Error>>
    {
        let response = client.get_websocket_session_url(&url)?;
        let (ws_stream, _) = connect(&response[0].web_socket_debugger_url)?;
        println!("WebSocket handshake has been successfully completed");

        Ok(ws_stream)
    }

    pub fn run_command(
        &mut self,
        command: &mut Value,
    ) -> Result<(), Box<dyn Error>>
    {
        command["id"] = 1.into();

        self.websocket
            .write_message(Message::Text(command.to_string()))?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct ChromeSession
{
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: String,
}

pub trait ChromeAPI
{
    fn get_websocket_session_url(
        &self,
        chrome_json_url: &String,
    ) -> Result<Vec<ChromeSession>, Box<dyn std::error::Error>>;
}

struct ChromeAPIClient;
impl ChromeAPI for ChromeAPIClient
{
    fn get_websocket_session_url(
        &self,
        chrome_json_url: &String,
    ) -> Result<Vec<ChromeSession>, Box<dyn std::error::Error>>
    {
        let response = reqwest::blocking::get(chrome_json_url)?.json::<Vec<ChromeSession>>()?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests
{
    use std::{net::TcpListener, thread::spawn, vec};

    use serde_json::{json, Value};
    use tungstenite::accept;

    use crate::{ChromeAPI, ChromeSession, ChromiumBrowser};

    struct MockChromeClient;
    impl ChromeAPI for MockChromeClient
    {
        fn get_websocket_session_url(
            &self,
            chrome_json_url: &String,
        ) -> Result<Vec<crate::ChromeSession>, Box<dyn std::error::Error>>
        {
            assert_eq!("http://test_url.com", chrome_json_url);
            Ok(vec![ChromeSession {
                web_socket_debugger_url: "ws://localhost:8081".to_string(),
            }])
        }
    }

    #[test]
    fn create_browser()
    {
        start_mock_server();

        let url = String::from("http://test_url.com");

        ChromiumBrowser::connect_with_client(&MockChromeClient {}, &url).unwrap();
    }

    #[test]
    fn run_command()
    {
        spawn(|| {
            let server = TcpListener::bind("127.0.0.1:8081").unwrap();
            for stream in server.incoming()
            {
                let mut web_socket = accept(stream.unwrap()).unwrap();

                let message = web_socket.read_message().unwrap();
                assert!(message.is_text());
                let data: Value = serde_json::from_str(&message.to_string()).unwrap();

                assert_eq!("Page.navigate", data["method"])
            }
        });

        let url = String::from("http://test_url.com"); 

        let mut browser = ChromiumBrowser::connect_with_client(&MockChromeClient {}, &url).unwrap();

        let mut message = json!({
            "id": 1, 
            "method": "Page.navigate", 
            "params": {
            "url": "https://google.com", 
            }
        });

        browser.run_command(&mut message).unwrap();
    }

    fn start_mock_server()
    {
        spawn(|| {
            let server = TcpListener::bind("127.0.0.1:8081").unwrap();
            for stream in server.incoming()
            {
                let mut web_socket = accept(stream.unwrap()).unwrap();
            }
        });
    }
}
