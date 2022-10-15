#[allow(dead_code, unused_imports, unused_variables, )]

use tokio_tungstenite::accept_async;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use std::{collections::HashSet, collections::HashMap, sync::{Arc, Mutex}, net::SocketAddr, io::Error as IoError};
use tokio::sync::mpsc;
use futures::{Stream, stream::TryStreamExt, StreamExt, channel::mpsc::{unbounded, UnboundedSender}, future, SinkExt, pin_mut, };

mod client;

#[derive(Eq, Hash, PartialEq, Clone)]
struct User {
    name: String,
    addr: SocketAddr 
}

type Tx = UnboundedSender<Message>;
type Degens = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[tokio::main]
async fn main() -> Result<(), IoError>{
    
    let peer_map = Degens::new(Mutex::new(HashMap::new()));

    let mut server: TcpListener = TcpListener::bind("127.0.0.1:6969").await.expect("unable to listen");

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

    let sending_to_others = andar.try_for_each(|msg| {
       println!("message received: {}", msg.to_text().unwrap()); 

       let peers = degens.lock().unwrap();

       let others_but_me = peers.iter().filter(|(others_addr, _)| others_addr != &&addr).map(|(_, ws_sink)| ws_sink);

       for chad in others_but_me {
           chad.unbounded_send(msg.clone()).unwrap();
       }

       future::ok(())
    });

    let recv_from_others = rx.map(Ok).forward(bahar);
    pin_mut!(sending_to_others, recv_from_others);
    future::select(sending_to_others, recv_from_others).await;
    
    println!("{} disconnected", &addr);

    degens.lock().unwrap().remove(&addr);
}


