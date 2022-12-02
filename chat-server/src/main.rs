use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    future, pin_mut,
    stream::TryStreamExt,
    SinkExt, Stream, StreamExt,
};
use std::{
    collections::HashMap,
    collections::HashSet,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
#[allow(dead_code, unused_imports, unused_variables)]
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

mod client;

#[derive(Eq, Hash, PartialEq, Clone)]
struct User {
    name: String,
    addr: SocketAddr,
}

type Tx = UnboundedSender<Message>;
type Degens = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let peer_map = Degens::new(Mutex::new(HashMap::new()));

    let mut server: TcpListener = TcpListener::bind("127.0.0.1:6969")
        .await
        .expect("unable to listen");

    while let Ok((stream, addr)) = server.accept().await {
        tokio::spawn(handle_me_daddy(peer_map.clone(), stream, addr));
    }
    Ok(())
}

async fn handle_me_daddy(degens: Degens, raw_stream: TcpStream, addr: SocketAddr) {
    let ws_stream = accept_async(raw_stream).await.expect("Failed ws handshake");
    println!("new degen connected, {}", addr);
    let (bahar, andar) = ws_stream.split();
    let (tx, rx) = unbounded::<Message>();

    degens.lock().unwrap().insert(addr, tx);

    //recv msg from the THE client
    let sending_to_others = andar.try_for_each(|msg| {
        println!("Message Received: {}", msg.to_text().unwrap());

        //check if ping here also
        //if ping, send ping to the user
        //if text, send text with addr
        let peers = degens.lock().unwrap();

        match msg {
            Message::Text(s) => {
                let others_but_me = peers
                    .iter()
                    .filter(|(others_addr, _)| others_addr != &&addr)
                    .map(|(_, ws_sink)| ws_sink);

                let mut usr_addr_str = addr.clone().to_string();
                usr_addr_str = String::from(format!("{} ", usr_addr_str));
                usr_addr_str.push_str(&s.to_string());
                //send msg + address of who sent it to everyone
                for chad in others_but_me {
                    chad.unbounded_send(Message::Text(usr_addr_str.clone()))
                        .unwrap();
                }
            }
            Message::Ping(p) => {
                //send to ping to who it's meant to ping
                let urs_addr: SocketAddr = String::from_utf8(p).unwrap().parse().unwrap();

                let usr_to_send_ack = peers
                    .iter()
                    .filter(|(ad, _)| ad == &&urs_addr)
                    .map(|(_, ws_sink)| ws_sink);

                for ping_user in usr_to_send_ack {
                    ping_user
                        // .unbounded_send(Message::Ping(String::from().into_bytes()))
                        .unbounded_send(Message::Ping(urs_addr.to_string().into_bytes()))
                        .unwrap();
                }
            }
            _ => (),
        }

        future::ok(())
    });

    let recv_from_others = rx
        .map(|msg| {
            let msg = match msg {
                Message::Ping(ack) => {
                    let ack_print = ack.clone();
                    println!("at recv from others{:?}", ack_print);
                    println!("{}", String::from_utf8(ack_print).unwrap());
                    let usr_addr: SocketAddr = String::from_utf8(ack).unwrap().parse().unwrap();
                    let peers = degens.lock().unwrap();

                    let usr_to_send_ack = peers
                        .iter()
                        .filter(|(others_addr, _)| others_addr == &&usr_addr)
                        .map(|(_, ws_sink)| ws_sink);

                    for usr in usr_to_send_ack {
                        usr.unbounded_send(Message::Pong(usr_addr.to_string().into_bytes()))
                            .unwrap();
                    }
                    // Message::Text(String::from(""))
                    Message::Ping(String::from("").into_bytes())
                }
                Message::Text(t) => Message::Text(t),
                Message::Binary(_) => todo!(),
                Message::Pong(p) => Message::Pong(p),
                Message::Close(_) => todo!(),
                Message::Frame(_) => todo!(),
            };
            return Ok(msg);
        })
        .forward(bahar);
    // let recv_from_others = rx.map(|what| what).forward(bahar);
    pin_mut!(sending_to_others, recv_from_others);
    future::select(sending_to_others, recv_from_others).await;

    println!("{} disconnected", &addr);

    degens.lock().unwrap().remove(&addr);
}
