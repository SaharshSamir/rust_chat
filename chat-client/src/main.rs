use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::{SinkExt, TryFutureExt};
use futures_util::{future, pin_mut, StreamExt};
use std::env;
use tokio::io::{stdin, stdout};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    //let _server_url  = env::args().nth(1).unwrap_or_else(|| panic!("pass in an argument"));

    let url = url::Url::parse("ws://127.0.0.1:6969").unwrap();

    let (stdin_tx, stdin_rx) = unbounded::<Message>();
    let tx = stdin_tx.clone();
    //spawn a thread to take in stdin and transmit it through the unbounded channel
    tokio::spawn(handle_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(url).await.expect("Cannot handshake");

    let (ws_write, mut ws_read) = ws_stream.split();

    // forward stdin to the ws sink
    let stdin_to_ws = stdin_rx.map(Ok).forward(ws_write);

    // put what we get through the ws, to stdout
    let ws_to_stdout = ws_read.for_each(|message| async {
        match message {
            Ok(m) => {
                println!("---------Debug--------> {}", m);
                match m {
                    Message::Text(msg) => {
                        // let data = msg;
                        // stdin_tx.unbounded_send(Message::Ping(vec![]));
                        let mut payload_iter = msg.splitn(2, " ");
                        let usr_addr = payload_iter.next();
                        let actual_message = payload_iter.next();
                        // let (tx, rx) = unbounded::<Message>();
                        match usr_addr {
                            Some(usr_address) => tx
                                .unbounded_send(Message::Ping(
                                    String::from(usr_address).into_bytes(),
                                ))
                                .unwrap(),
                            None => (),
                        }

                        match actual_message {
                            Some(m) => println!("{}", m),
                            None => (),
                        }
                        // tx.unbounded_send(Message::Ping(usr_addr.));
                        // rx.map(Ok).forward(ws_write);
                        // tokio::spawn(future);
                        // stdout().write_all(&data).await.unwrap();
                    }
                    //recv ack
                    Message::Pong(_) => {
                        println!("✅");
                    }
                    _ => {
                        print!("");
                    }
                }
            }
            Err(e) => println!("{:?}", e.to_string()),
        }
    });

    // loop {
    //     tokio::select! {
    //         msg = ws_read.recv().next() {

    //         }
    //     }
    // }

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

// async fn handle_ack(tx: UnboundedSender<Message>) {
//     tx.unbound
// }

//read data from the stdin and send it along the UnboundedSender tx
async fn handle_stdin(tx: UnboundedSender<Message>) {
    let mut stdin = stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);

        tx.unbounded_send(Message::Text(String::from_utf8(buf).unwrap()))
            .unwrap();
    }
}
// ✅
