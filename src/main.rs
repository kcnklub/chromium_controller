use core::panic;
use std::{str};

use futures_util::{future, pin_mut, StreamExt};
use json::object;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main()
{
    let response = reqwest::get("http://localhost:8080/json")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let response_json = json::parse(&response).expect("unable to parse google response");

    let session_url = response_json[0]["webSocketDebuggerUrl"].as_str().unwrap();

    let chrome_url = url::Url::parse(session_url).unwrap();

    let (sender, receiver) = futures_channel::mpsc::unbounded();

    let performance_enabled = object! {
        id: 1,
        method: "Performance.enable",
        params: object! {
            timeDomain: "timeTicks",
        }
    };
    sender
        .unbounded_send(Message::Text(String::from(performance_enabled.dump())))
        .expect("couldn't send page enable message");

    let page_enable = object! {
        id: 2,
        method: "Page.enable", 
    };
    sender.unbounded_send(Message::Text(String::from(page_enable.dump()))).expect("couldn't send page enable message");
    tokio::spawn(read_stdin(sender));

    let (ws_stream, _) = connect_async(chrome_url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = receiver.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = match message
            {
                Ok(some) => some.into_data(),
                Err(error) => panic!("{error}"),
            };
            let data_string = str::from_utf8(&data).unwrap();
            let response = json::parse(data_string).unwrap();
            println!("Response: {}", response.dump());
            tokio::io::stdout().write_all(&data).await.unwrap();
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(sender: futures_channel::mpsc::UnboundedSender<Message>)
{
    let mut counter = 3;
    let mut stdin = tokio::io::stdin();
    loop
    {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await
        {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };

        buf.truncate(n);

        let url = str::from_utf8(&buf).unwrap();

        let data = object! {
            id: 3,
            method: "Page.navigate",
            params: object! {
                url: url
            }
        };
        counter = counter + 1;
        println!("{}", data.dump());
        sender
            .unbounded_send(Message::Text(String::from(data.dump())))
            .unwrap();
    }
}
