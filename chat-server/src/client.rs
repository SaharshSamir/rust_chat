use std::env;
use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let something = "breathe air";
    let url = env::args().nth(1).unwrap_or_else(|| panic!("poop"));
    
    let connection = connect_async(url);

}

