use tokio_tungstenite;
use futures_util::io::AsyncReadExt;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt,SinkExt};
use serde_json::{from_str, Value};
#[tokio::main]
async fn main() {
    let (mut ws,_)= tokio_tungstenite::connect_async("ws://api.testnet.solana.com/").await.expect("failed to connect to ws");
    println!("connected");
    let (mut w, mut r) =ws.split();
    let msg = Message::Text(r#"{ "jsonrpc": "2.0", "id": 1, "method": "slotSubscribe" }"#.into());
    w.send(msg).await.expect("failed to send json");
    loop{
        if let Some(m) = r.next().await{
            let m = m.expect("failed to read message");
            let m = m.into_text().expect("failed to convert to a string");
            let m = m.as_str();
            let v:Value = serde_json::from_str(&m).expect("cannot unpack");
            println!("{}",v["params"]["result"]["slot"]); 
        }        
    }

}
