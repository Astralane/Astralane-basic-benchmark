use jito_geyser_protos::solana::geyser::{geyser_client::GeyserClient,SubscribeSlotUpdateRequest};
use tonic::transport::Endpoint;
use tonic::{Status};
use tokio_tungstenite;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt,SinkExt};
use serde_json;
use serde::{Serialize,Deserialize};
use tokio::sync::mpsc;

#[derive(Debug)]
struct Msg{

    key:String,
    value: u64

}


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
    // println!("{m}");    
    let v:SolanaApiOutput = serde_json::from_str(&m).expect("cannot unpack");
    let mut starting_slot:u64= v.params.result.slot as u64; 
    starting_slot+=100;
    println!("staring at slot {starting_slot}");
    // creating channel for mpsc
    let (tx, mut rx) = mpsc::channel(16);
    let tx2 = tx.clone();

    let mut handles = vec![];
    let handle2=tokio::spawn(
        async move{
            grpc_slots(starting_slot,tx2).await;
        });
    handles.push(handle2);
    let handle1=tokio::spawn(
        async move{
            ws_slots(starting_slot,tx).await;
        });
    handles.push(handle1);
    let mut final_slots: Vec<u64> = Vec::with_capacity(100);
    let mut ws_score =0;
    let mut grpc_score = 0;
    while let Some(message) = rx.recv().await{
        if message.key.as_str()=="grpc" && !final_slots.contains(&message.value){
            grpc_score+=1;
            final_slots.push(message.value);
        }else if message.key.as_str()=="ws" && !final_slots.contains(&message.value){
            ws_score+=1;
            final_slots.push(message.value);
        }
    }                  
    println!(" ws {}%, grpc {}%, equal {}% \n ", ws_score ,grpc_score ,100-(ws_score+grpc_score));       
    for handle in handles{
        handle.await.unwrap();
    }

}

async fn ws_slots(ss:u64, tx:tokio::sync::mpsc::Sender<Msg>){
   let (ws,_)= tokio_tungstenite::connect_async("ws://api.testnet.solana.com/").await.expect("failed to connect to ws");
    println!("connected to ws");
    let (mut w, mut r) =ws.split();
    let inp_req = SolanaApi{
        jsonrpc: "2.0".to_string(),
        id: 2,
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
    loop{

        let m=r.next().await.unwrap();

        let m = m.expect("failed to read message");
        let m = m.into_text().expect("failed to convert to a string");
        let m = m.as_str();    
        let v:SolanaApiOutput = serde_json::from_str(&m).expect("cannot unpack");
        let slot_num_ws=v.params.result.slot as u64;
        if slot_num_ws == ss{
            break;
        }        
        let mm = Msg {
            key:"ws".to_string(),
            value:slot_num_ws,
        };
        tx.send(mm).await.expect("ws multi threading crashed");

    }

}

async fn grpc_slots(ss:u64,tx:tokio::sync::mpsc::Sender<Msg>){
    let endpoint = Endpoint::from_static("http://198.244.253.220:10000");
    let channel = endpoint.connect().await.expect("connect");
    let mut client = GeyserClient::with_interceptor(channel, intrcptr);
    let mut stream = client.subscribe_slot_updates(SubscribeSlotUpdateRequest {}).await.expect("couldn't get stream").into_inner();
    loop{
       
        
        let slot_grpc=stream.message().await.expect("not getting slot update from grpc");

         // current slot from grpc
        
        let slot_up=slot_grpc.unwrap().slot_update.unwrap();
        let slot_num_grpc=slot_up.slot;
        if slot_num_grpc == ss {
            break
        }
        if slot_up.status == 1{
            let mm = Msg {
                key:"grpc".to_string(),
                value:slot_num_grpc,
            };
            tx.send(mm).await.expect("grpc multi threading crashed");

        }


    }    
}
fn intrcptr(request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        // request.metadata_mut();
//            .insert("access-token", self.access_token.parse().unwrap());
        Ok(request)
}