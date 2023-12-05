//! Tests for the `ciphers_nft` contract.
use ciphers_nft::{
  contract_view::*,
  events::{ContractEvent, DeployEvent},
  getters::*,
  init::InitParams,
  mint::*,
};
use concordium_cis2::*;
use concordium_smart_contract_testing::*;

use super::init::*;

// Helper function that mints a token to the given address.
pub fn mint_to_address(
  chain: &mut Chain,
  contract_address: ContractAddress,
  mint_params: MintParams,
  invoker: Option<AccountAddress>,
  sender: Option<Address>,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
  let invoker = invoker.unwrap_or(MINTER);
  let sender = sender.unwrap_or(MINTER_ADDR);

  let update_result = chain.contract_update(
    SIGNER,
    invoker,
    sender,
    Energy::from(10000),
    UpdateContractPayload {
      amount: Amount::zero(),
      receive_name: OwnedReceiveName::new_unchecked("ciphers_nft.mint".to_string()),
      address: contract_address,
      message: OwnedParameter::from_serial(&mint_params).expect("Mint params"),
    },
  );

  update_result
}

/// Setup chain and contract.
pub fn initialize_chain_and_contract(timestamp: u64) -> (Chain, ContractAddress) {
  let mut chain = Chain::builder()
    .block_time(Timestamp::from_timestamp_millis(timestamp))
    .build()
    .unwrap();

  // Create some accounts accounts on the chain.
  chain.create_account(Account::new(OWNER, ACC_INITIAL_BALANCE));
  chain.create_account(Account::new(MINTER, ACC_INITIAL_BALANCE));
  chain.create_account(Account::new(USER, ACC_INITIAL_BALANCE));
  chain.create_account(Account::new(USER2, ACC_INITIAL_BALANCE));
  chain.create_account(Account::new(USER3, ACC_INITIAL_BALANCE));
  chain.create_account(Account::new(NEW_MINTER, ACC_INITIAL_BALANCE));

  // Load and deploy the module.
  let module = module_load_v1("ciphers_nft.wasm.v1").expect("Module exists");
  let deployment = chain
    .module_deploy_v1(SIGNER, OWNER, module)
    .expect("Deploy valid module");

  let params = InitParams {
    name: NAME.to_string(),
    symbol: SYMBOL.to_string(),
    contract_uri: get_contract_metadata(),
    minter: MINTER,
    mint_start: MINT_START,
    mint_deadline: MINT_DEADLINE,
    max_total_supply: MAX_TOTAL_SUPPLY,
  };

  // Initialize the auction contract.
  let init = chain
    .contract_init(
      SIGNER,
      OWNER,
      Energy::from(10000),
      InitContractPayload {
        amount: Amount::zero(),
        mod_ref: deployment.module_reference,
        init_name: OwnedContractName::new_unchecked("init_ciphers_nft".to_string()),
        param: OwnedParameter::from_serial(&params).expect("Init params"),
      },
    )
    .expect("Initialize contract");

  for event in init.events {
    let contract_event = event.parse::<ContractEvent>().expect("Deserialize event");
    // println!("Event: {:?}", contract_event);

    assert_eq!(
      contract_event,
      ContractEvent::Deploy(DeployEvent {
        name: NAME.to_string(),
        symbol: SYMBOL.to_string(),
        contract_uri: get_contract_metadata(),
        minter: MINTER,
        mint_start: MINT_START,
        mint_deadline: MINT_DEADLINE,
        max_total_supply: MAX_TOTAL_SUPPLY,
      })
    );
  }

  (chain, init.contract_address)
}

pub fn get_view_state(chain: &Chain, contract_address: ContractAddress) -> ViewState {
  let invoke = chain
    .contract_invoke(
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("ciphers_nft.view".to_string()),
        address: contract_address,
        message: OwnedParameter::empty(),
      },
    )
    .expect("Invoke view");

  invoke.parse_return_value().expect("ViewState return value")
}

pub fn get_view_address(
  chain: &Chain,
  contract_address: ContractAddress,
  address: Address,
) -> ViewAddress {
  let invoke = chain
    .contract_invoke(
      USER,
      USER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("ciphers_nft.viewAddress".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&ContractViewAddressQueryParams { address })
          .expect("ViewAddress params"),
      },
    )
    .expect("Invoke view");

  invoke
    .parse_return_value()
    .expect("ViewAddress return value")
}

#[allow(unused)]
pub fn get_view_settings(chain: &Chain, contract_address: ContractAddress) -> ViewSettings {
  let invoke = chain
    .contract_invoke(
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("ciphers_nft.viewSettings".to_string()),
        address: contract_address,
        message: OwnedParameter::empty(),
      },
    )
    .expect("Invoke view");

  invoke.parse_return_value().expect("ViewState return value")
}

#[allow(unused)]
pub fn c_mint_params(token: u32) -> MintParams {
  MintParams {
    owners: vec![USER_ADDR],
    tokens: vec![TokenIdU32(token)],
    token_uris: vec!["ipfs://test".to_string()],
  }
}

pub fn get_contract_metadata() -> MetadataUrl {
  MetadataUrl {
    url: "ipfs://contractURI".to_string(),
    hash: None,
  }
}
