use jito_geyser_protos::solana::geyser::{
    geyser_client::GeyserClient,SubscribeSlotUpdateRequest
};
use tonic::{
    transport::Endpoint,
};
use tonic::{Status};
use tokio_tungstenite;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt,SinkExt};
use serde_json;
use serde::{Serialize,Deserialize};
use std::cmp::Ordering;

#[derive(Serialize)]
struct SolanaApi{
    jsonrpc: String,
    id: i32,
    method: String
}

#[derive(Deserialize)]
struct SolanaApiOutput{
    params: SolanaApiResult,
}

#[derive(Deserialize)]
struct SolanaApiResult{
    result: SolanaApiSlot,
}
#[derive(Deserialize)]
struct SolanaApiSlot{
    slot:i32,
}

// #[derive(Deserialize)]
// struct slot_grpc{

// }


#[tokio::main]

async fn main() {
    let endpoint = Endpoint::from_static("http://198.244.253.220:10000");

    println!("connected to grpc");
    let (ws,_)= tokio_tungstenite::connect_async("ws://api.testnet.solana.com/").await.expect("failed to connect to ws");
    println!("connected to ws");
    let (mut w, mut r) =ws.split();
    let inp_req = SolanaApi{
        jsonrpc: "2.0".to_string(),
        id: 1,
        method:"slotSubscribe".to_string()
    };
    let msg = Message::Text(serde_json::to_string(&inp_req).expect("couldn't convert to "));
    w.send(msg).await.expect("failed to send json");
    //confirmation
    let m=r.next().await.unwrap();
    let m = m.expect("failed to read message");
    let m = m.into_text().expect("failed to convert to a string");
    let m = m.as_str();
    println!("{m}");
    //actual message
    let m=r.next().await.unwrap();
    let m = m.expect("failed to read message");
    let m = m.into_text().expect("failed to convert to a string");
    let m = m.as_str();
    println!("{m}");    
    let v:SolanaApiOutput = serde_json::from_str(&m).expect("cannot unpack");
    let mut starting_slot:u64= v.params.result.slot as u64; 
    starting_slot+=100;
    println!("staring at slot {starting_slot}");   
    let channel = endpoint.connect().await.expect("connect");
    let mut client = GeyserClient::with_interceptor(channel, intrcptr);
    
    loop{
       


        //current slot from ws
        let m=r.next().await.unwrap();
        let m = m.expect("failed to read message");
        let m = m.into_text().expect("failed to convert to a string");
        let m = m.as_str();    
        let v:SolanaApiOutput = serde_json::from_str(&m).expect("cannot unpack");
        let slot_num_ws=v.params.result.slot as u64;
         // current slot from grpc
        let mut stream = client.subscribe_slot_updates(SubscribeSlotUpdateRequest {}).await.expect("couldn't get stream").into_inner();
        let slot_grpc=stream.message().await.expect("not getting slot update from grpc");
        
        let slot_up=slot_grpc.unwrap().slot_update.unwrap();
        let slot_num_grpc=slot_up.slot;        
        if slot_up.status == 1{
            match slot_num_grpc.cmp(&slot_num_ws){
                Ordering::Less => println!("our validator is losing by {} slot_ws : {slot_num_ws} slot_grpc: {slot_num_grpc}",slot_num_ws-slot_num_grpc),
                Ordering::Greater => println!("our validator is winning by {}",slot_num_grpc-slot_num_ws),
                Ordering::Equal => println!("they are both equal"),

            }
           
        }
        if slot_num_grpc==starting_slot || slot_num_ws == starting_slot {
            break;
        }
    }

}
 fn intrcptr(mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        request.metadata_mut();
//            .insert("access-token", self.access_token.parse().unwrap());
        Ok(request)
}