use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::system_transaction::transfer;
fn main (){
    let url = "https://api.testnet.solana.com".to_string();
    let client = RpcClient::new(url);
    let account:Pubkey = "3C8zyv56HuXZnaUqmFkSSsx3ZL1ioADJHtuGQpkQByAh".parse().expect("invalid");//receiver
    let balance=client.get_balance(&account).expect("cannot get balance");
    println!("{}",balance);
    let priv_key=keypair::read_keypair_file("/Users/dev77/Desktop/solana-test/test-account.json");
    let priv_key:Keypair= match priv_key {
        Ok(x) => x,
        _ => Keypair::new(),
    };
    let latest_hash = client.get_latest_blockhash().expect("couldn't get lastest hash");
    let tx= transfer(&priv_key,&account,1000,latest_hash);
    let signature= client.send_and_confirm_transaction(&tx).expect("FAILED TO SEND TRANSACTION");
    println!("SUCCESSFULLY TRANSFER SOL https://explorer.solana.com/tx/{}?cluster=testnet",signature);

    //todo add grpc reads below 
}