//! A NFT smart contract example using the Concordium Token Standard CIS2.
//!
//! # Description
//! An instance of this smart contract can contain a number of different token
//! each identified by a token ID. A token is then globally identified by the
//! contract address together with the token ID.
//!
//! In this example the contract is initialized with no tokens, and tokens can
//! be minted through a `mint` contract function, which will only succeed for
//! the contract owner. No functionality to burn token is defined in this
//! example.
//!
//! Note: The word 'address' refers to either an account address or a
//! contract address.
//!
//! As follows from the CIS2 specification, the contract has a `transfer`
//! function for transferring an amount of a specific token type from one
//! address to another address. An address can enable and disable one or more
//! addresses as operators. An operator of some address is allowed to transfer
//! any tokens owned by this address.
//!
//! Tests are located in `./tests/tests.rs`.

use concordium_cis2::*;
use concordium_std::*;

use crate::{
  error::{ContractError, ContractResult},
  state::State,
};

/// List of supported standards by this contract address.
pub const SUPPORTS_STANDARDS: [StandardIdentifier<'static>; 2] =
  [CIS0_STANDARD_IDENTIFIER, CIS2_STANDARD_IDENTIFIER];

// Types

/// Contract token ID type.
/// To save bytes we use a token ID type limited to a `u32`.
pub type ContractTokenId = TokenIdU32;
pub type MintCountTokenID = u32;

/// Contract token amount.
/// Since the tokens are non-fungible the total supply of any token will be at
/// most 1 and it is fine to use a small type for representing token amounts.
pub type ContractTokenAmount = TokenAmountU8;

/// The parameter type for the contract function `setImplementors`.
/// Takes a standard identifier and list of contract addresses providing
/// implementations of this standard.
#[derive(Debug, Serialize, SchemaType)]
pub struct SetImplementorsParams {
  /// The identifier for the standard.
  pub id: StandardIdentifierOwned,
  /// The addresses of the implementors of the standard.
  pub implementors: Vec<ContractAddress>,
}

type TransferParameter = TransferParams<ContractTokenId, ContractTokenAmount>;

/// Execute a list of token transfers, in the order of the list.
///
/// Logs a `Transfer` event and invokes a receive hook function for every
/// transfer in the list.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - Any of the transfers fail to be executed, which could be if:
///     - The `token_id` does not exist.
///     - The sender is not the owner of the token, or an operator for this
///       specific `token_id` and `from` address.
///     - The token is not owned by the `from`.
/// - Fails to log event.
/// - Any of the receive hook function calls rejects.
#[receive(
  contract = "test_nft",
  name = "transfer",
  parameter = "TransferParameter",
  error = "ContractError",
  enable_logger,
  mutable
)]
fn contract_transfer(
  ctx: &ReceiveContext,
  host: &mut Host<State>,
  logger: &mut Logger,
) -> ContractResult<()> {
  // Parse the parameter.
  let TransferParams(transfers): TransferParameter = ctx.parameter_cursor().get()?;
  // Get the sender who invoked this contract function.
  let sender = ctx.sender();

  for Transfer {
    token_id,
    amount,
    from,
    to,
    data,
  } in transfers
  {
    let (state, builder) = host.state_and_builder();
    // Authenticate the sender for this transfer
    ensure!(
      from == sender || state.is_operator(&sender, &from),
      ContractError::Unauthorized
    );
    let to_address = to.address();
    // Update the contract state
    state.transfer(&token_id, amount, &from, &to_address, builder)?;

    // Log transfer event
    logger.log(&Cis2Event::Transfer(TransferEvent {
      token_id,
      amount,
      from,
      to: to_address,
    }))?;

    // If the receiver is a contract: invoke the receive hook function.
    if let Receiver::Contract(address, function) = to {
      let parameter = OnReceivingCis2Params {
        token_id,
        amount,
        from,
        data,
      };
      host.invoke_contract(
        &address,
        &parameter,
        function.as_entrypoint_name(),
        Amount::zero(),
      )?;
    }
  }
  Ok(())
}

/// Enable or disable addresses as operators of the sender address.
/// Logs an `UpdateOperator` event.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - Fails to log event.
#[receive(
  contract = "test_nft",
  name = "updateOperator",
  parameter = "UpdateOperatorParams",
  error = "ContractError",
  enable_logger,
  mutable
)]
fn contract_update_operator(
  ctx: &ReceiveContext,
  host: &mut Host<State>,
  logger: &mut Logger,
) -> ContractResult<()> {
  // Parse the parameter.
  let UpdateOperatorParams(params) = ctx.parameter_cursor().get()?;
  // Get the sender who invoked this contract function.
  let sender = ctx.sender();
  let (state, builder) = host.state_and_builder();
  for param in params {
    // Update the operator in the state.
    match param.update {
      OperatorUpdate::Add => state.add_operator(&sender, &param.operator, builder),
      OperatorUpdate::Remove => state.remove_operator(&sender, &param.operator),
    }

    // Log the appropriate event
    logger.log(
      &Cis2Event::<ContractTokenId, ContractTokenAmount>::UpdateOperator(UpdateOperatorEvent {
        owner: sender,
        operator: param.operator,
        update: param.update,
      }),
    )?;
  }

  Ok(())
}

/// Takes a list of queries. Each query is an owner address and some address to
/// check as an operator of the owner address.
///
/// It rejects if:
/// - It fails to parse the parameter.
#[receive(
  contract = "test_nft",
  name = "operatorOf",
  parameter = "OperatorOfQueryParams",
  return_value = "OperatorOfQueryResponse",
  error = "ContractError"
)]
fn contract_operator_of(
  ctx: &ReceiveContext,
  host: &Host<State>,
) -> ContractResult<OperatorOfQueryResponse> {
  // Parse the parameter.
  let params: OperatorOfQueryParams = ctx.parameter_cursor().get()?;
  // Build the response.
  let mut response = Vec::with_capacity(params.queries.len());
  for query in params.queries {
    // Query the state for address being an operator of owner.
    let is_operator = host.state().is_operator(&query.address, &query.owner);
    response.push(is_operator);
  }
  let result = OperatorOfQueryResponse::from(response);
  Ok(result)
}

/// Parameter type for the CIS-2 function `balanceOf` specialized to the subset
/// of TokenIDs used by this contract.
type ContractBalanceOfQueryParams = BalanceOfQueryParams<ContractTokenId>;
/// Response type for the CIS-2 function `balanceOf` specialized to the subset
/// of TokenAmounts used by this contract.
type ContractBalanceOfQueryResponse = BalanceOfQueryResponse<ContractTokenAmount>;

/// Get the balance of given token IDs and addresses.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - Any of the queried `token_id` does not exist.
#[receive(
  contract = "test_nft",
  name = "balanceOf",
  parameter = "ContractBalanceOfQueryParams",
  return_value = "ContractBalanceOfQueryResponse",
  error = "ContractError"
)]
fn contract_balance_of(
  ctx: &ReceiveContext,
  host: &Host<State>,
) -> ContractResult<ContractBalanceOfQueryResponse> {
  // Parse the parameter.
  let params: ContractBalanceOfQueryParams = ctx.parameter_cursor().get()?;
  // Build the response.
  let mut response = Vec::with_capacity(params.queries.len());
  for query in params.queries {
    // Query the state for balance.
    let amount = host.state().balance(&query.token_id, &query.address)?;
    response.push(amount);
  }
  let result = ContractBalanceOfQueryResponse::from(response);
  Ok(result)
}

/// Parameter type for the CIS-2 function `tokenMetadata` specialized to the
/// subset of TokenIDs used by this contract.
pub type ContractTokenMetadataQueryParams = TokenMetadataQueryParams<ContractTokenId>;

/// Get the token metadata URLs and checksums given a list of token IDs.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - Any of the queried `token_id` does not exist.
#[receive(
  contract = "test_nft",
  name = "tokenMetadata",
  parameter = "ContractTokenMetadataQueryParams",
  return_value = "TokenMetadataQueryResponse",
  error = "ContractError"
)]
fn contract_token_metadata(
  ctx: &ReceiveContext,
  host: &Host<State>,
) -> ContractResult<TokenMetadataQueryResponse> {
  // Parse the parameter.
  let params: ContractTokenMetadataQueryParams = ctx.parameter_cursor().get()?;
  // Build the response.
  let mut response = Vec::with_capacity(params.queries.len());
  for token_id in params.queries {
    // Check the token exists.
    ensure!(
      host.state().contains_token(&token_id),
      ContractError::InvalidTokenId
    );
    let token_uri = host
      .state()
      .token_uris
      .get(&token_id)
      .ok_or(ContractError::InvalidTokenId)?;

    let metadata_url = MetadataUrl {
      url: token_uri.to_string(),
      hash: None,
    };
    response.push(metadata_url);
  }
  let result = TokenMetadataQueryResponse::from(response);
  Ok(result)
}

/// Get the supported standards or addresses for a implementation given list of
/// standard identifiers.
///
/// It rejects if:
/// - It fails to parse the parameter.
#[receive(
  contract = "test_nft",
  name = "supports",
  parameter = "SupportsQueryParams",
  return_value = "SupportsQueryResponse",
  error = "ContractError"
)]
fn contract_supports(
  ctx: &ReceiveContext,
  host: &Host<State>,
) -> ContractResult<SupportsQueryResponse> {
  // Parse the parameter.
  let params: SupportsQueryParams = ctx.parameter_cursor().get()?;

  // Build the response.
  let mut response = Vec::with_capacity(params.queries.len());
  for std_id in params.queries {
    if SUPPORTS_STANDARDS.contains(&std_id.as_standard_identifier()) {
      response.push(SupportResult::Support);
    } else {
      response.push(host.state().have_implementors(&std_id));
    }
  }
  let result = SupportsQueryResponse::from(response);
  Ok(result)
}

/// Set the addresses for an implementation given a standard identifier and a
/// list of contract addresses.
///
/// It rejects if:
/// - Sender is not the owner of the contract instance.
/// - It fails to parse the parameter.
#[receive(
  contract = "test_nft",
  name = "setImplementors",
  parameter = "SetImplementorsParams",
  error = "ContractError",
  mutable
)]
fn contract_set_implementor(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
  // Authorize the sender.
  ensure!(
    ctx.sender().matches_account(&ctx.owner()),
    ContractError::Unauthorized
  );
  // Parse the parameter.
  let params: SetImplementorsParams = ctx.parameter_cursor().get()?;
  // Update the implementors in the state
  host
    .state_mut()
    .set_implementors(params.id, params.implementors);
  Ok(())
}

//////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Serialize, SchemaType)]
#[concordium(transparent)]
pub struct MintCountQueryParams<T: IsTokenId> {
  /// List of balance queries.
  #[concordium(size_length = 2)]
  pub queries: Vec<T>,
}

pub type ContractMintCountQueryParams = MintCountQueryParams<ContractTokenId>;

#[derive(Debug, Serialize, SchemaType)]
#[concordium(transparent)]
pub struct TokenMintCountQueryResponse(#[concordium(size_length = 2)] pub Vec<MintCountTokenID>);

impl From<Vec<MintCountTokenID>> for TokenMintCountQueryResponse {
  fn from(results: Vec<MintCountTokenID>) -> Self {
    TokenMintCountQueryResponse(results)
  }
}

#[receive(
  contract = "test_nft",
  name = "getMintCountTokenID",
  parameter = "ContractMintCountQueryParams",
  return_value = "TokenMintCountQueryResponse",
  error = "ContractError"
)]
fn contract_get_mint_count_token_id(
  ctx: &ReceiveContext,
  host: &Host<State>,
) -> ContractResult<TokenMintCountQueryResponse> {
  // Parse the parameter.
  let params: ContractMintCountQueryParams = ctx.parameter_cursor().get()?;
  // Build the response.
  let mut response = Vec::with_capacity(params.queries.len());
  for token_id in params.queries {
    // Check the token exists.
    ensure!(
      host.state().contains_token(&token_id),
      ContractError::InvalidTokenId
    );
    let mint_count = host
      .state()
      .mint_count
      .get(&token_id)
      .ok_or(ContractError::InvalidTokenId)?;

    response.push(*mint_count);
  }
  let result = TokenMintCountQueryResponse::from(response);
  Ok(result)
}

#[derive(Serialize, SchemaType, Debug)]
pub struct ViewSettings {
  pub minter: AccountAddress,
  pub mint_start: u64,
  pub mint_deadline: u64,
  pub max_total_supply: u32,
}

#[receive(
  contract = "test_nft",
  name = "viewSettings",
  return_value = "ViewSettings"
)]
fn contract_view_settings(
  _ctx: &ReceiveContext,
  host: &Host<State>,
) -> ReceiveResult<ViewSettings> {
  let state = host.state();

  Ok(ViewSettings {
    minter: state.minter,
    mint_start: state.mint_start,
    mint_deadline: state.mint_deadline,
    max_total_supply: state.max_total_supply,
  })
}

#[derive(Debug, Serialize, SchemaType)]
pub struct SetMinter {
  pub minter: AccountAddress,
}

#[receive(
  contract = "test_nft",
  name = "setMinter",
  parameter = "SetMinter",
  error = "ContractError",
  mutable
)]
fn contract_set_minter(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
  ensure!(
    ctx.sender().matches_account(&ctx.owner()),
    ContractError::Unauthorized
  );

  let params: SetMinter = ctx.parameter_cursor().get()?;
  host.state_mut().set_minter(params.minter);
  Ok(())
}
