use concordium_cis2::*;
use concordium_std::*;

use crate::{
  cis2::{ContractTokenAmount, ContractTokenId},
  error::{ContractError, ContractResult, CustomContractError},
  events::{ContractEvent, MintedEvent},
  state::State,
};

/// The parameter for the contract function `mint` which mints a number of
/// tokens to a given address.
#[derive(Serial, Deserial, SchemaType)]
pub struct MintParams {
  /// Owner of the newly minted tokens.
  #[concordium(size_length = 1)] // max size of 256
  pub owners: Vec<Address>,
  /// A collection of tokens to mint.
  #[concordium(size_length = 1)] // max size of 256
  pub tokens: Vec<ContractTokenId>,
  /// The metadata URL for the token.
  #[concordium(size_length = 1)] // max size of 256
  pub token_uris: Vec<String>,
}

/// Mint new tokens with a given address as the owner of these tokens.
/// Can only be called by the contract owner.
/// Logs a `Mint` and a `TokenMetadata` event for each token.
/// The url for the token metadata is the token ID encoded in hex, appended on
/// the `TOKEN_METADATA_BASE_URL`.
///
/// It rejects if:
/// - The sender is not the contract instance owner.
/// - Fails to parse parameter.
/// - Any of the tokens fails to be minted, which could be if:
///     - The minted token ID already exists.
///     - Fails to log Mint event
///     - Fails to log TokenMetadata event
///
/// Note: Can at most mint 32 token types in one call due to the limit on the
/// number of logs a smart contract can produce on each function call.
#[receive(
  contract = "test_nft",
  name = "mint",
  parameter = "MintParams",
  error = "ContractError",
  enable_logger,
  mutable
)]
fn contract_mint(
  ctx: &ReceiveContext,
  host: &mut Host<State>,
  logger: &mut Logger,
) -> ContractResult<()> {
  let (state, builder) = host.state_and_builder();
  let sender = ctx.sender();
  let minter = state.minter;
  ensure!(sender.matches_account(&minter), ContractError::Unauthorized);
  // Get the sender of the transaction
  let block_time: u64 = ctx.metadata().block_time().timestamp_millis();
  ensure!(
    block_time >= state.mint_start,
    CustomContractError::MintingNotStarted.into()
  );
  ensure!(
    block_time < state.mint_deadline,
    CustomContractError::MintDeadlineReached.into()
  );

  // Parse the parameter.
  let params: MintParams = ctx.parameter_cursor().get()?;
  for ((&token_id, owner), token_uri) in params
    .tokens
    .iter()
    .zip(params.owners)
    .zip(params.token_uris)
  {
    // Mint the token in the state.
    let mint_count = state.mint(token_id, &owner, &token_uri, builder)?;

    // Event for minted NFT.
    logger.log(&ContractEvent::Mint(MintEvent {
      token_id,
      amount: ContractTokenAmount::from(1),
      owner,
    }))?;

    // Metadata URL for the NFT.
    // ADD COUNTER AND Timestamp mayber REMOVE?
    logger.log(&ContractEvent::TokenMetadata(TokenMetadataEvent {
      token_id,
      metadata_url: MetadataUrl {
        url: token_uri.clone(),
        hash: None,
      },
    }))?;

    // Event for minted NFT.
    logger.log(&ContractEvent::Minted(MintedEvent {
      token_id,
      mint_count,
      timestamp: block_time,
      token_uri: MetadataUrl {
        url: token_uri,
        hash: None,
      },
    }))?;
  }

  Ok(())
}
