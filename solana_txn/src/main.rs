use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::system_transaction::transfer;
use jito_geyser_protos::solana::geyser::{
    geyser_client::GeyserClient,SubscribeTransactionUpdatesRequest,
};
use tonic::{
    transport::Endpoint,
};
use tonic::{Status};
use solana_sdk::signature::Signature;
use std::time::{SystemTime};

#[tokio::main]
async fn main (){
    let url = "https://api.testnet.solana.com".to_string();
    let client = RpcClient::new(url);
    let account:Pubkey = "3C8zyv56HuXZnaUqmFkSSsx3ZL1ioADJHtuGQpkQByAh".parse().expect("invalid");//receiver
    let balance=client.get_balance(&account).expect("cannot get balance");
    println!("{}",balance);
    let priv_key=keypair::read_keypair_file("/home/d_e/Desktop/solana-testnet/id.json");
    let priv_key:Keypair= match priv_key {
        Ok(x) => x,
        _ => Keypair::new(),
    };
    let latest_hash = client.get_latest_blockhash().expect("couldn't get lastest hash");
    let tx= transfer(&priv_key,&account,1000,latest_hash);
    let signature= client.send_transaction(&tx).expect("FAILED TO SEND TRANSACTION");
    println!("SUCCESSFULLY TRANSFER SOL https://explorer.solana.com/tx/{}?cluster=testnet",signature);
    let duration_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    println!("sent at {duration_since_epoch:?}");
    //todo add grpc reads below 

    let endpoint = Endpoint::from_static("http://198.244.253.220:10000");
    let channel = endpoint.connect().await.expect("connect");
    let mut client = GeyserClient::with_interceptor(channel, intrcptr);
    let mut response = client.subscribe_transaction_updates(SubscribeTransactionUpdatesRequest {}).await.expect("cannot get response from grpc txn sub").into_inner();
    loop {
        let transaction_update = response.message().await.expect("not getting transaction update");
        match transaction_update{
            None => {println!("error receiving update in loop");}
            Some(transaction_update)=>{
               println!("output: {transaction_update:?}");
                if let Some(ts) = transaction_update.ts{
                    println!("{ts:?}");
                }
                let a =transaction_update.transaction;
                match a{
                    None => {println!("error receiving update in loop");}
                    Some(a) => {
                        // println!("FINAL: {:?}",a.signature);
                        let finalsig:Signature = a.signature.parse().unwrap();
                        // let finalsig=finalsig.from_str(a.signature);
                        if finalsig == signature {
                            let duration_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                            println!("found sig {signature} at slot {} at timestamp {duration_since_epoch:?}", a.slot);
                            break;
                        }
                    }
                }
                
            }
        }
    }
}

 fn intrcptr(mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        request.metadata_mut();
//            .insert("access-token", self.access_token.parse().unwrap());
        Ok(request)
    }