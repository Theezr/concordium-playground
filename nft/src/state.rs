use concordium_cis2::*;
use concordium_std::*;

use crate::{
  cis2::{ContractTokenAmount, ContractTokenId, MintCountTokenID},
  error::{ContractError, ContractResult, CustomContractError},
  init::InitParams,
};

/// The state for each address.
#[derive(Serial, DeserialWithState, Deletable)]
#[concordium(state_parameter = "S")]
pub struct AddressState<S = StateApi> {
  /// The tokens owned by this address.
  pub owned_tokens: StateSet<ContractTokenId, S>,
  /// The address which are currently enabled as operators for this address.
  pub operators: StateSet<Address, S>,
}

impl AddressState {
  fn empty(state_builder: &mut StateBuilder) -> Self {
    AddressState {
      owned_tokens: state_builder.new_set(),
      operators: state_builder.new_set(),
    }
  }
}

/// The contract state.
// Note: The specification does not specify how to structure the contract state
// and this could be structured in a more space efficient way depending on the use case.
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S = StateApi> {
  pub name: String,
  pub symbol: String,
  /// The state for each address.
  pub address_state: StateMap<Address, AddressState<S>, S>,
  /// All of the token IDs
  pub all_tokens: StateSet<ContractTokenId, S>,
  /// Map with the tokenUris
  pub token_uris: StateMap<ContractTokenId, String, S>,
  /// Map with contract addresses providing implementations of additional
  /// standards.
  pub implementors: StateMap<StandardIdentifierOwned, Vec<ContractAddress>, S>,
  /// address of the minter
  pub minter: AccountAddress,
  /// Counter of the mints
  pub counter: MintCountTokenID,
  /// Counter of the mint
  pub mint_count: StateMap<ContractTokenId, MintCountTokenID, S>,
  /// Unix timestamp to start minting
  pub mint_start: u64,
  /// Minting deadline in Unix timestamp
  pub mint_deadline: u64,
  /// Max total supply
  pub max_total_supply: u32,
}

impl State {
  /// Creates a new state with no tokens.
  pub fn init(state_builder: &mut StateBuilder, init_params: InitParams) -> Self {
    State {
      name: init_params.name,
      symbol: init_params.symbol,
      address_state: state_builder.new_map(),
      all_tokens: state_builder.new_set(),
      token_uris: state_builder.new_map(),
      implementors: state_builder.new_map(),
      mint_count: state_builder.new_map(),
      counter: 0,
      minter: init_params.minter,
      mint_start: init_params.mint_start,
      mint_deadline: init_params.mint_deadline,
      max_total_supply: init_params.max_total_supply,
    }
  }

  /// Mint a new token with a given address as the owner
  pub fn mint(
    &mut self,
    token: ContractTokenId,
    owner: &Address,
    token_uri: &String,
    state_builder: &mut StateBuilder,
  ) -> ContractResult<u32> {
    ensure!(
      self.all_tokens.insert(token) && self.token_uris.insert(token, token_uri.clone()).is_none(),
      CustomContractError::TokenIdAlreadyExists.into()
    );

    self.counter += 1;
    let count = self.counter;

    ensure!(
      count <= self.max_total_supply,
      CustomContractError::MaxTotalSupplyReached.into()
    );

    self.mint_count.insert(token, count);

    let mut owner_state = self
      .address_state
      .entry(*owner)
      .or_insert_with(|| AddressState::empty(state_builder));

    owner_state.owned_tokens.insert(token);

    Ok(count)
  }

  /// Check that the token ID currently exists in this contract.
  #[inline(always)]
  pub fn contains_token(&self, token_id: &ContractTokenId) -> bool {
    self.all_tokens.contains(token_id)
  }

  /// Get the current balance of a given token ID for a given address.
  /// Results in an error if the token ID does not exist in the state.
  /// Since this contract only contains NFTs, the balance will always be
  /// either 1 or 0.
  pub fn balance(
    &self,
    token_id: &ContractTokenId,
    address: &Address,
  ) -> ContractResult<ContractTokenAmount> {
    ensure!(self.contains_token(token_id), ContractError::InvalidTokenId);
    let balance = self
      .address_state
      .get(address)
      .map(|address_state| u8::from(address_state.owned_tokens.contains(token_id)))
      .unwrap_or(0);
    Ok(balance.into())
  }

  /// Check if a given address is an operator of a given owner address.
  pub fn is_operator(&self, address: &Address, owner: &Address) -> bool {
    self
      .address_state
      .get(owner)
      .map(|address_state| address_state.operators.contains(address))
      .unwrap_or(false)
  }

  /// Update the state with a transfer of some token.
  /// Results in an error if the token ID does not exist in the state or if
  /// the from address have insufficient tokens to do the transfer.
  pub fn transfer(
    &mut self,
    token_id: &ContractTokenId,
    amount: ContractTokenAmount,
    from: &Address,
    to: &Address,
    state_builder: &mut StateBuilder,
  ) -> ContractResult<()> {
    ensure!(self.contains_token(token_id), ContractError::InvalidTokenId);
    // A zero transfer does not modify the state.
    if amount == 0.into() {
      return Ok(());
    }
    // Since this contract only contains NFTs, no one will have an amount greater
    // than 1. And since the amount cannot be the zero at this point, the
    // address must have insufficient funds for any amount other than 1.
    ensure_eq!(amount, 1.into(), ContractError::InsufficientFunds);

    {
      let mut from_address_state = self
        .address_state
        .get_mut(from)
        .ok_or(ContractError::InsufficientFunds)?;
      // Find and remove the token from the owner, if nothing is removed, we know the
      // address did not own the token..
      let from_had_the_token = from_address_state.owned_tokens.remove(token_id);
      ensure!(from_had_the_token, ContractError::InsufficientFunds);
    }

    // Add the token to the new owner.
    let mut to_address_state = self
      .address_state
      .entry(*to)
      .or_insert_with(|| AddressState::empty(state_builder));
    to_address_state.owned_tokens.insert(*token_id);
    Ok(())
  }

  /// Update the state adding a new operator for a given address.
  /// Succeeds even if the `operator` is already an operator for the
  /// `address`.
  pub fn add_operator(
    &mut self,
    owner: &Address,
    operator: &Address,
    state_builder: &mut StateBuilder,
  ) {
    let mut owner_state: OccupiedEntry<'_, Address, AddressState, ExternStateApi> = self
      .address_state
      .entry(*owner)
      .or_insert_with(|| AddressState::empty(state_builder));
    owner_state.operators.insert(*operator);
  }

  /// Update the state removing an operator for a given address.
  /// Succeeds even if the `operator` is _not_ an operator for the `address`.
  pub fn remove_operator(&mut self, owner: &Address, operator: &Address) {
    self
      .address_state
      .entry(*owner)
      .and_modify(|address_state| {
        address_state.operators.remove(operator);
      });
  }

  /// Check if state contains any implementors for a given standard.
  pub fn have_implementors(&self, std_id: &StandardIdentifierOwned) -> SupportResult {
    if let Some(addresses) = self.implementors.get(std_id) {
      SupportResult::SupportBy(addresses.to_vec())
    } else {
      SupportResult::NoSupport
    }
  }

  /// Set implementors for a given standard.
  pub fn set_implementors(
    &mut self,
    std_id: StandardIdentifierOwned,
    implementors: Vec<ContractAddress>,
  ) {
    self.implementors.insert(std_id, implementors);
  }

  pub fn set_minter(&mut self, minter: AccountAddress) {
    self.minter = minter;
  }
}
