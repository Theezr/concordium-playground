//! Test the `GetBlockTransactionEvents` endpoint.
use anyhow::Context;
use concordium_rust_sdk::{
  common::deserial_bytes,
  smart_contracts::common::Serialize,
  types::{AbsoluteBlockHeight, BlockItemSummaryDetails, ContractAddress},
  v2::{self, Endpoint},
};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Amount {
  micro_ccd: u64,
}

#[derive(Debug, Deserialize)]
struct AccountAddress(Vec<u8>);

#[derive(Debug, Deserialize)]
struct ContractEvent {
  bytes: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct OwnedParameter(Vec<u8>);

#[derive(Debug, Deserialize)]
struct OwnedReceiveName(String);

#[derive(Debug, Deserialize)]
struct InstanceUpdatedEvent {
  contract_version: String,
  address: ContractAddress,
  instigator: AccountAddress,
  amount: Amount,
  message: OwnedParameter,
  receive_name: OwnedReceiveName,
  events: Vec<ContractEvent>,
}

#[derive(Debug, Deserialize)]
struct Updated {
  data: InstanceUpdatedEvent,
}

#[derive(Debug, Deserialize)]
struct ContractUpdateIssued {
  effects: Vec<Updated>,
}

#[derive(Debug, Deserialize)]
struct AccountTransactionDetails {
  cost: Amount,
  sender: AccountAddress,
  effects: ContractUpdateIssued,
}

#[derive(Debug, Deserialize)]
struct AccountTransaction {
  details: AccountTransactionDetails,
}

struct App {
  endpoint: v2::Endpoint,
  height: AbsoluteBlockHeight,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let app = {
    let app = App {
      endpoint: Endpoint::from_static("http://node.testnet.concordium.com:20000"),
      height: AbsoluteBlockHeight::from(7_921_000),
    };
    App::from(app)
  };

  let mut client = v2::Client::new(app.endpoint)
    .await
    .context("Cannot connect.")?;

  println!("Getting finalized blocks from {}.", app.height);

  let mut receiver = client.get_finalized_blocks_from(app.height).await?;
  while let Some(v) = receiver.next().await {
    let bi = client.get_block_info(v.block_hash).await?;
    if bi.response.transaction_count > 0 {
      let mut events = client
        .get_block_transaction_events(v.block_hash)
        .await?
        .response;
      while let Some(event) = events.next().await.transpose()? {
        if event
          .affected_contracts()
          .contains(&ContractAddress::new(7418, 0))
        {
          let details_value = event.details.clone();

          println!("Event {:?}", event.details.clone());

          println!(
            "Transaction {} with sender {}.",
            &event.hash,
            event.sender_account().unwrap()
          );
        }
      }
    }
  }
  Ok(())
}
