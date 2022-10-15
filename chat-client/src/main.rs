use std::env;
use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::channel::mpsc::{unbounded, UnboundedSender};
use tokio::io::{stdin, stdout};

#[tokio::main]
async fn main() {

    //let _server_url  = env::args().nth(1).unwrap_or_else(|| panic!("pass in an argument"));

    let url = url::Url::parse("ws://127.0.0.1:6969").unwrap();

    let (stdin_tx, stdin_rx) = unbounded::<Message>();
    //spawn a thread to take in stdin and transmit it through the unbounded channel
    tokio::spawn(handle_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(url).await.expect("Cannot handshake");

    let (ws_write, ws_read) = ws_stream.split();
    
    //forward stdin to the ws sink
    let stdin_to_ws = stdin_rx.map(Ok).forward(ws_write);

    //put what we get through the ws, to stdout
    let ws_to_stdout = ws_read.for_each(|message| {
        async {
            let data = message.unwrap().into_data();
            stdout().write_all(&data).await.unwrap();
        }
    });

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

//read data from the stdin and send it along the UnboundedSender tx 
async fn handle_stdin(tx: UnboundedSender<Message>) {
    let mut stdin = stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n
        };
        buf.truncate(n);

        tx.unbounded_send(Message::binary(buf)).unwrap();
    }

}



