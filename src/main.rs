use core::panic;
use std::{str};

use futures_util::{future, pin_mut, StreamExt};
use json::object;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main()
{
    let chrome_url = url::Url::parse(
        "ws://127.0.0.1:8080/devtools/browser/ca96cad9-d247-42a1-bca7-410de7d464a3",
    )
    .unwrap();
    // let chrome_url = url::Url::parse("ws://localhost:3000").unwrap();

    let (sender, receiver) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(sender));

    let (ws_stream, _) = connect_async(chrome_url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = receiver.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            println!("{:?}", message);
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
            id: 1,
            method: "Page.navigate",
            params: object! {
                url: url
            }
        };
        println!("{}", data.dump());
        sender
            .unbounded_send(Message::Text(String::from(data.dump())))
            .unwrap();
    }
}
