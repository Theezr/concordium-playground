//! Test the `GetBlockTransactionEvents` endpoint.
use anyhow::Context;
use concordium_rust_sdk::{
  cis2::{TokenAmount, TokenId},
  contract_client::MetadataUrl,
  smart_contracts::common::{AccountAddress, Cursor, Get, ParseError, ParseResult, Read},
  types::smart_contracts::concordium_contracts_common::Deserial,
  types::{
    smart_contracts::ContractEvent, AbsoluteBlockHeight, AccountTransactionEffects, Address,
    BlockItemSummaryDetails, ContractAddress, ContractTraceElement, InstanceUpdatedEvent,
  },
  v2::{self, Endpoint},
};
use futures::StreamExt;
use hex;
use serde::Deserialize;

#[derive(Debug)]
pub struct MintEvent {
  pub token_id: TokenId,
  pub amount: TokenAmount,
  // pub owner: AccountAddress,
}

impl Deserial for MintEvent {
  fn deserial<R: Read>(source: &mut R) -> ParseResult<Self> {
    let token_id: TokenId = source.get()?;
    let amount: TokenAmount = source.get()?;

    println!("token_id: {:?}", token_id);
    println!("amount: {:?}", amount);

    Ok(MintEvent { token_id, amount })
  }
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
          let events: Vec<ContractEvent> = event
            .contract_update_logs()
            .unwrap()
            .flat_map(|(_, events)| events)
            .skip(1)
            .cloned()
            .collect();

          println!("EVENTS \n {:?}", events);

          for event in events {
            println!("EVENT \n {}", event.to_string());
            let test: MintEvent = event.parse()?;
            println!("{:?}", test);
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
