//! Test the `GetBlockTransactionEvents` endpoint.
use anyhow::Context;
use concordium_rust_sdk::{
  common::deserial_bytes,
  smart_contracts::common::Serialize,
  types::{
    smart_contracts::ContractEvent, AbsoluteBlockHeight, BlockItemSummaryDetails, ContractAddress,
    InstanceUpdatedEvent,
  },
  v2::{self, Endpoint},
};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::Value;

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
          let details_value: BlockItemSummaryDetails = event.details.clone();
          // Extract ContractEvents from AccountTransaction
          let contract_events = extract_contract_events(&details_value);

          println!("Event {:?}", details_value);

          // println!(
          //   "Transaction {} with sender {}.",
          //   &event.hash,
          //   event.sender_account().unwrap()
          // );
        }
      }
    }
  }
  Ok(())
}

fn extract_contract_events(account_transaction: &BlockItemSummaryDetails) -> Vec<ContractEvent> {
  if let InstanceUpdatedEvent { events, .. } = &account_transaction.details.effects {
    // Access the events field within InstanceUpdatedEvent
    return events.clone();
  }
  // Return an empty vector if the structure doesn't match the expected pattern
  vec![]
}
