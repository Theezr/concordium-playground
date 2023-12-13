//! Test the `GetBlockTransactionEvents` endpoint.
use anyhow::Context;
use concordium_rust_sdk::{
  smart_contracts::common::{Deserial, ParseError, Read, AccountAddress},
  types::{
    smart_contracts::ContractEvent, AbsoluteBlockHeight, BlockItemSummaryDetails, ContractAddress,
    InstanceUpdatedEvent, ContractTraceElement, AccountTransactionEffects, Address,
  },
  v2::{self, Endpoint}, contract_client::MetadataUrl, common::{ParseResult, deserial_bytes}, cis2::{TokenId, TokenAmount},
};
use futures::StreamExt;
use serde::Deserialize;
use hex; 

#[derive(Deserialize, Debug)]
pub struct MintEvent {
  pub token_id: u32,
  pub amount: u64,
  pub owner: AccountAddress,
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
          let events: Vec<ContractEvent> = event.contract_update_logs().unwrap().flat_map(|(_, events)| events).cloned().collect();
          
          println!("EVENTS \n {:?}", events);

          for event in events {
            println!("EVENT \n {}", event.to_string());


        match deserialize_event(&event) {
          Ok(mint_event) => {
              println!("MINT EVENT \n {:?}", mint_event);
          }
          Err(e) => {
              eprintln!("Failed to deserialize event: {:?}", e);
          }
      }
        }

        

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

fn deserialize_event(hex_str: &str) -> Result<MintEvent, bincode::Error> {
  let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
  bincode::deserialize(&bytes)
}