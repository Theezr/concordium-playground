use concordium_cis2::*;
use concordium_std::*;

use crate::{
  cis2::{ContractTokenId, MintCountTokenID},
  error::{ContractError, ContractResult, CustomContractError},
  state::State,
};

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
  contract = "ciphers_nft",
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
  pub name: String,
  pub symbol: String,
  pub contract_uri: MetadataUrl,
  pub minter: AccountAddress,
  pub mint_start: u64,
  pub mint_deadline: u64,
  pub max_total_supply: u32,
}

#[receive(
  contract = "ciphers_nft",
  name = "viewSettings",
  return_value = "ViewSettings"
)]
fn contract_view_settings(
  _ctx: &ReceiveContext,
  host: &Host<State>,
) -> ReceiveResult<ViewSettings> {
  let state = host.state();

  Ok(ViewSettings {
    name: state.name.clone(),
    symbol: state.symbol.clone(),
    contract_uri: state.contract_uri.clone(),
    minter: state.minter,
    mint_start: state.mint_start,
    mint_deadline: state.mint_deadline,
    max_total_supply: state.max_total_supply,
  })
}

#[derive(Serialize, SchemaType, PartialEq, Eq, Debug)]
pub struct ViewAddress {
  pub owned_tokens: Vec<ContractTokenId>,
  pub operators: Vec<Address>,
}

#[derive(Debug, Serialize, SchemaType)]
#[concordium(transparent)]
pub struct ContractViewAddressQueryParams {
  pub address: Address,
}

#[receive(
  contract = "ciphers_nft",
  name = "viewAddress",
  return_value = "ViewAddress"
)]
fn contract_view_address(ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<ViewAddress> {
  let state = host.state();
  let address = ctx.sender();
  let a_state = state
    .address_state
    .get(&address)
    .ok_or(CustomContractError::InvalidAddress)?;

  let owned_tokens = a_state.owned_tokens.iter().map(|x| *x).collect();
  let operators = a_state.operators.iter().map(|x| *x).collect();

  Ok(ViewAddress {
    owned_tokens,
    operators,
  })
}
