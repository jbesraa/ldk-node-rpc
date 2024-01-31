extern crate lazy_static;
use std::collections::HashMap;

use ldk_node::bitcoin::Network;
use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::lightning::ln::ChannelId;
use lightning::ChanDetails;
use tonic::{transport::Server, Request, Response, Status};
use walletrpc::wallet_rpc_service_server::{WalletRpcService, WalletRpcServiceServer};
use walletrpc::{
    Channel, CloseChannelRequest, ConnectToPeerRequest, CreateInvoiceReply, CreateInvoiceRequest,
    DisconnectPeerRequest, GeneralNodeNameRequest, GeneralSuccessReply, GetNodeIdReply,
    GetOnChainAddressReply, GetOnChainBalanceReply, ListChannelsReply, ListPeersReply,
    NewWalletReply, NewWalletRequest, OpenChannelRequest, PayInvoiceRequest, Peer, StartNodeReply,
	GetEsploraAddressReply, GetNetAddressReply,
};

use crate::lightning::WrappedPeerDetails;
pub mod lightning;
pub mod paths;
pub mod wallet;

pub mod walletrpc {
    // The string specified here must match the proto package name
    tonic::include_proto!("walletrpc");
}

#[derive(Debug, Default)]
pub struct MyWallet {}

#[tonic::async_trait]
impl WalletRpcService for MyWallet {
    async fn new_wallet(
        &self,
        request: Request<NewWalletRequest>,
    ) -> Result<Response<NewWalletReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let mnemonic = wallet::Wallet::new(
            Network::Testnet,
            &request.wallet_name,
            &request.listening_address,
            &request.esplora_address,
        )
        .unwrap();
        let reply = NewWalletReply {
            mnemonic: mnemonic.to_string(),
        };
        Ok(Response::new(reply))
    }

    async fn start_node(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<StartNodeReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::start_node(node_name);
        let reply = StartNodeReply {
            success: response.0,
            msg: response.1,
        };
        Ok(Response::new(reply))
    }

    async fn is_node_running(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<GeneralSuccessReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::is_node_running(node_name);
        let reply = GeneralSuccessReply { success: response };
        Ok(Response::new(reply))
    }

    async fn get_node_id(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<GetNodeIdReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::get_node_id(node_name);
        let reply = GetNodeIdReply { node_id: response };
        Ok(Response::new(reply))
    }

    async fn stop_node(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<GeneralSuccessReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::stop_node(node_name);
        let reply = GeneralSuccessReply { success: response };
        Ok(Response::new(reply))
    }

    async fn pay_invoice(
        &self,
        request: Request<PayInvoiceRequest>,
    ) -> Result<Response<GeneralSuccessReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.our_node_name;
        let invoice = request.invoice;
        let response = lightning::pay_invoice(node_name, invoice);
        let success = if let Some(_r) = response { true } else { false };
        let reply = GeneralSuccessReply { success };
        Ok(Response::new(reply))
    }

    async fn get_on_chain_address(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<GetOnChainAddressReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::new_onchain_address(node_name);
        let reply = GetOnChainAddressReply { address: response };
        Ok(Response::new(reply))
    }

    async fn get_on_chain_balance(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<GetOnChainBalanceReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::total_onchain_balance(node_name);
        let reply = GetOnChainBalanceReply {
            balance: response as i64,
        };
        Ok(Response::new(reply))
    }

    async fn create_invoice(
        &self,
        request: Request<CreateInvoiceRequest>,
    ) -> Result<Response<CreateInvoiceReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let our_node_name = request.our_node_name;
        let amount_msat = request.amount_msat;
        let description = request.description;
        let expiry_secs = request.expiry_secs;
        let response = lightning::create_invoice(
            our_node_name,
            amount_msat as u64,
            &description,
            expiry_secs as u32,
        )
        .unwrap();
        let reply = CreateInvoiceReply { invoice: response };
        Ok(Response::new(reply))
    }

    async fn close_channel(
        &self,
        request: Request<CloseChannelRequest>,
    ) -> Result<Response<GeneralSuccessReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let our_node_name = request.our_node_name;
        let node_id = request.node_id;
        use ldk_node::lightning::ln::ChannelId;
        let channel_id = request.channel_id.clone().into_bytes()[..]
            .try_into()
            .unwrap();
        let channel_id = ChannelId(channel_id);
        let response = lightning::close_channel(our_node_name, node_id, channel_id);
        let reply = GeneralSuccessReply { success: response };
        Ok(Response::new(reply))
    }

    async fn open_channel(
        &self,
        request: Request<OpenChannelRequest>,
    ) -> Result<Response<GeneralSuccessReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let our_node_name = request.our_node_name;
        let node_id = request.node_id;
        let net_address = request.net_address;
        let channel_amount_sats = request.channel_amount_sats;
        let push_to_counterparty_msat = request.push_to_counterparty_msat;
        let announce_channel = request.announce_channel;
        let response = lightning::open_channel(
            our_node_name,
            node_id,
            net_address,
            channel_amount_sats as u64,
            push_to_counterparty_msat as u64,
            announce_channel,
        );
        let reply = GeneralSuccessReply { success: response };
        Ok(Response::new(reply))
    }

    async fn list_peers(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<ListPeersReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::list_peers(node_name);
        let peers: HashMap<i32, Peer> = response
            .into_iter()
            .enumerate()
            .map(|(k, v)| (k as i32, v.into()))
            .collect::<HashMap<i32, Peer>>();
        let reply = ListPeersReply { peers };
        Ok(Response::new(reply))
    }

    async fn list_channels(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<ListChannelsReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::list_channels(node_name);
        let channels: HashMap<i32, Channel> = response
            .into_iter()
            .enumerate()
            .map(|(k, v)| (k as i32, v.into()))
            .collect::<HashMap<i32, Channel>>();
        let reply = ListChannelsReply { channels };
        Ok(Response::new(reply))
    }

    async fn disconnect_peer(
        &self,
        request: Request<DisconnectPeerRequest>,
    ) -> Result<Response<GeneralSuccessReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let our_node_name = request.our_node_name;
        let node_id = request.node_id;
        let response = lightning::disconnect_peer(our_node_name, node_id);
        let reply = GeneralSuccessReply { success: response };
        Ok(Response::new(reply))
    }

    async fn connect_to_peer(
        &self,
        request: Request<ConnectToPeerRequest>,
    ) -> Result<Response<GeneralSuccessReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let our_node_name = request.our_node_name;
        let node_id = request.node_id;
        let net_address = request.net_address;
        let response = lightning::connect_to_node(our_node_name, node_id, net_address);
        let reply = GeneralSuccessReply { success: response };
        Ok(Response::new(reply))
    }

    async fn get_esplora_address(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<GetEsploraAddressReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::get_esplora_address(node_name);
        let reply = GetEsploraAddressReply { address: response };
        Ok(Response::new(reply))
    }

    async fn get_net_address(
        &self,
        request: Request<GeneralNodeNameRequest>,
    ) -> Result<Response<GetNetAddressReply>, Status> {
        println!("Got a request: {:?}", request);
        let request = request.into_inner();
        let node_name = request.node_name;
        let response = lightning::get_our_address(node_name);
        let reply = GetNetAddressReply { address: response };
        Ok(Response::new(reply))
    }
}

impl From<ChanDetails> for Channel {
    fn from(chan_details: ChanDetails) -> Self {
        Channel {
            channel_id: chan_details.channel_id.to_string(),
            counterparty_node_id: chan_details.counterparty_node_id.to_string(),
            channel_value_sats: chan_details.channel_value_sats as i64,
            balance_msat: chan_details.balance_msat as i64,
            outbound_capacity_msat: chan_details.outbound_capacity_msat as i64,
            inbound_capacity_msat: chan_details.inbound_capacity_msat as i64,
            is_outbound: chan_details.is_outbound,
            is_channel_ready: chan_details.is_channel_ready,
            is_usable: chan_details.is_usable,
            is_public: chan_details.is_public,
        }
    }
}

// impl From<Channel> for ChanDetails {
//     fn from(channel: Channel) -> Self {
//         ChanDetails {
//             channel_id: ChannelId(channel.channel_id),
//             counterparty_node_id: PublicKey::from_slice(&channel.counterparty_node_id.as_bytes()[..]).unwrap(),
//             channel_value_sats: channel.channel_value_sats as u64,
//             balance_msat: channel.balance_msat as u64,
//             outbound_capacity_msat: channel.outbound_capacity_msat as i64,
//             inbound_capacity_msat: channel.inbound_capacity_msat as i64,
//             is_outbound: channel.is_outbound,
//             is_channel_ready: channel.is_channel_ready,
//             is_usable: channel.is_usable,
//             is_public: channel.is_public,
//         }
//     }
// }


impl From<WrappedPeerDetails> for Peer {
    fn from(peer_details: WrappedPeerDetails) -> Self {
        Peer {
            address: peer_details.address,
            node_id: peer_details.node_id.to_string(),
            is_persisted: peer_details.is_persisted,
            is_connected: peer_details.is_connected,
            alias: peer_details.alias,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyWallet::default();
    Server::builder()
        .add_service(WalletRpcServiceServer::new(greeter))
        .serve(addr)
        .await?;
    Ok(())
}
