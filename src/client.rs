// use crate::walletrpc::{IsNodeRunningRequest, StartNodeRequest};
// use walletrpc::wallet_rpc_service_client::WalletRpcServiceClient;

// pub mod walletrpc {
//     tonic::include_proto!("walletrpc");
// }

// pub async fn start_node(req: tonic::Request<StartNodeRequest>) -> Result<(), Box<dyn std::error::Error>> {
//     let mut client = WalletRpcServiceClient::connect("http://[::1]:50051").await?;
//     let response = client.start_node(req).await?;

//     println!("RESPONSE={:?}", response);
//     Ok(())
// }

// pub async fn is_node_running(req: tonic::Request<IsNodeRunningRequest> ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut client = WalletRpcServiceClient::connect("http://[::1]:50051").await?;
//     let response = client.is_node_running(req).await?;

//     println!("RESPONSE={:?}", response);
//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut client = WalletRpcServiceClient::connect("http://[::1]:50051").await?;
    // let req = tonic::Request::new(StartNodeRequest {
    //     node_name: "oip".into(),
    // });
    // let response = client.start_node(req).await?;

    // println!("RESPONSE={:?}", response);

    Ok(())
}
