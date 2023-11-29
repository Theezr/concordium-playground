//! Tests for the `test_nft` contract.
use concordium_cis2::*;
use concordium_smart_contract_testing::*;
use concordium_std::concordium_test;
use test_nft::{
  cis2::*,
  contract_view::*,
  events::{ContractEvent, DeployEvent, MintedEvent},
  getters::*,
  init::*,
  mint::*,
  setters::*,
};

/// The tests accounts.
const OWNER: AccountAddress = AccountAddress([1; 32]);
const OWNER_ADDR: Address = Address::Account(OWNER);
const MINTER: AccountAddress = AccountAddress([2; 32]);
const MINTER_ADDR: Address = Address::Account(MINTER);
const USER: AccountAddress = AccountAddress([3; 32]);
const USER_ADDR: Address = Address::Account(USER);
const USER2: AccountAddress = AccountAddress([4; 32]);
const USER2_ADDR: Address = Address::Account(USER2);
const USER3: AccountAddress = AccountAddress([5; 32]);
const USER3_ADDR: Address = Address::Account(USER3);
const NEW_MINTER: AccountAddress = AccountAddress([6; 32]);

/// Token IDs.
const TOKEN_0: ContractTokenId = TokenIdU32(2);
const TOKEN_1: ContractTokenId = TokenIdU32(42);

/// Initial balance of the accounts.
const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(10000);

/// A signer for all the transactions.
const SIGNER: Signer = Signer::with_one_key();

const NAME: &str = "test nft contract";
const SYMBOL: &str = "TST";
const MINT_START: u64 = 100;
const MINT_DEADLINE: u64 = 1000;
const MAX_TOTAL_SUPPLY: u32 = 10;

/// Test minting succeeds and the tokens are owned by the given address and
/// the appropriate events are logged.
/// Also tests that the mint count for the token and the counter are updated.
#[concordium_test]
fn test_minting() {
  let chain_timestamp = MINT_START + 1;
  let (mut chain, contract_address) = initialize_chain_and_contract(chain_timestamp);

  let update = mint_to_address(&mut chain, contract_address, c_mint_params(2), None, None)
    .expect("Mint failed");

  // Check that the tokens are owned by Alice.
  let rv: ViewState = get_view_state(&chain, contract_address);
  // println!("rv: {:?}", rv);

  assert_eq!(rv.all_tokens[..], [TokenIdU32(2)]);
  assert_eq!(
    rv.state,
    vec![(
      USER_ADDR,
      ViewAddressState {
        owned_tokens: vec![TokenIdU32(2)],
        operators: Vec::new(),
      }
    )]
  );

  assert_eq!(rv.name, NAME);
  assert_eq!(rv.symbol, SYMBOL);
  assert_eq!(rv.contract_uri, get_contract_metadata());
  assert_eq!(rv.mint_count, vec![(TokenIdU32(2), 1)]);
  assert_eq!(rv.counter, 1);
  assert_eq!(rv.mint_start, MINT_START);
  assert_eq!(rv.mint_deadline, MINT_DEADLINE);
  assert_eq!(rv.max_total_supply, MAX_TOTAL_SUPPLY);

  let events = update.events().flat_map(|(_addr, events)| events);
  let events: Vec<ContractEvent> = events
    .map(|e| e.parse().expect("Deserialize event"))
    .collect();

  // println!("events: {:?}", events);

  assert_eq!(
    events,
    [
      ContractEvent::Mint(MintEvent {
        token_id: TokenIdU32(2),
        amount: TokenAmountU8(1),
        owner: USER_ADDR,
      }),
      ContractEvent::Minted(MintedEvent {
        token_id: TokenIdU32(2),
        mint_count: 1,
        timestamp: MINT_START + 1,
        token_uri: MetadataUrl {
          url: "ipfs://test".to_string(),
          hash: None,
        },
      })
    ]
  );
}

#[concordium_test]
fn test_batch_minting() {
  let chain_timestamp = MINT_START + 1;
  let (mut chain, contract_address) = initialize_chain_and_contract(chain_timestamp);

  let mint_params = MintParams {
    owners: vec![USER_ADDR, USER2_ADDR, USER3_ADDR],
    tokens: vec![TokenIdU32(2), TokenIdU32(20), TokenIdU32(200)],
    token_uris: vec![
      "ipfs://test".to_string(),
      "ipfs://test1".to_string(),
      "ipfs://test2".to_string(),
    ],
  };
  mint_to_address(&mut chain, contract_address, mint_params, None, None).expect("Mint failed");

  // Check that the tokens are owned by Alice.
  let rv: ViewState = get_view_state(&chain, contract_address);
  // println!("rv: {:?}", rv);

  assert_eq!(
    rv.all_tokens[..],
    [TokenIdU32(2), TokenIdU32(20), TokenIdU32(200)]
  );
  assert_eq!(
    rv.state,
    vec![
      (
        USER_ADDR,
        ViewAddressState {
          owned_tokens: vec![TokenIdU32(2)],
          operators: Vec::new(),
        }
      ),
      (
        USER2_ADDR,
        ViewAddressState {
          owned_tokens: vec![TokenIdU32(20)],
          operators: Vec::new(),
        }
      ),
      (
        USER3_ADDR,
        ViewAddressState {
          owned_tokens: vec![TokenIdU32(200)],
          operators: Vec::new(),
        }
      )
    ]
  );
  assert_eq!(
    rv.mint_count,
    vec![
      (TokenIdU32(2), 1),
      (TokenIdU32(20), 2),
      (TokenIdU32(200), 3)
    ]
  );
  assert_eq!(rv.counter, 3);
}

#[concordium_test]
fn test_token_metadata_on_mint() {
  let (mut chain, contract_address) = initialize_chain_and_contract(100);
  mint_to_address(&mut chain, contract_address, c_mint_params(2), None, None).expect("Mint failed");

  let token_ids = ContractTokenMetadataQueryParams {
    queries: vec![TOKEN_0],
  };

  // Invoke the view entrypoint and check that the tokens are owned by Alice.
  let invoke = chain
    .contract_invoke(
      USER,
      USER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.tokenMetadata".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&token_ids).expect("tokenIds params"),
      },
    )
    .expect("Invoke view");

  // Check that the tokenUri is set correctly
  let rv: TokenMetadataQueryResponse = invoke.parse_return_value().expect("ViewState return value");
  let TokenMetadataQueryResponse(urls) = rv;

  // println!("rv: {:?}", urls);

  assert_eq!(
    urls,
    vec![MetadataUrl {
      url: "ipfs://test".to_string(),
      hash: None,
    }]
  );
}

#[concordium_test]
fn test_get_mint_count_token_id() {
  let (mut chain, contract_address) = initialize_chain_and_contract(100);
  mint_to_address(&mut chain, contract_address, c_mint_params(2), None, None).expect("Mint failed");

  mint_to_address(&mut chain, contract_address, c_mint_params(42), None, None)
    .expect("Mint failed");

  let token_ids = ContractMintCountQueryParams {
    queries: vec![TOKEN_0, TOKEN_1],
  };

  // Invoke the view entrypoint and check that the tokens are owned by Alice.
  let invoke = chain
    .contract_invoke(
      USER,
      USER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.getMintCountTokenID".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&token_ids).expect("tokenIds params"),
      },
    )
    .expect("Invoke view");

  // Check that the tokenUri is set correctly
  let rv: TokenMintCountQueryResponse =
    invoke.parse_return_value().expect("ViewState return value");
  let TokenMintCountQueryResponse(counts) = rv;

  // println!("rv get mint count: {:?}", counts);

  assert_eq!(counts, vec![1, 2]);
}

#[concordium_test]
fn test_mint_should_fail_when_minting_not_started() {
  let chain_timestamp = MINT_START - 1;
  let (mut chain, contract_address) = initialize_chain_and_contract(chain_timestamp);

  let update_result = mint_to_address(&mut chain, contract_address, c_mint_params(2), None, None);

  assert!(update_result.is_err(), "Call didnt fail");
}

#[concordium_test]
fn test_mint_should_fail_when_mint_deadline_reached() {
  let chain_timestamp = MINT_DEADLINE + 1;
  let (mut chain, contract_address) = initialize_chain_and_contract(chain_timestamp);

  let update_result = mint_to_address(&mut chain, contract_address, c_mint_params(2), None, None);

  assert!(update_result.is_err(), "Call didnt fail");
}

#[concordium_test]
fn test_mint_should_fail_when_max_supply_reached() {
  let chain_timestamp = MINT_START + 1;
  let (mut chain, contract_address) = initialize_chain_and_contract(chain_timestamp);

  for i in 1..MAX_TOTAL_SUPPLY + 2 {
    let update_result = mint_to_address(&mut chain, contract_address, c_mint_params(i), None, None);

    if i <= MAX_TOTAL_SUPPLY {
      assert!(update_result.is_ok(), "Call didnt succeed");
    } else {
      assert!(update_result.is_err(), "Call didnt fail");
    }
  }
  // Handle update_result...
}

#[concordium_test]
fn test_contract_view_settings() {
  let chain_timestamp = MINT_START + 1;
  let (chain, contract_address) = initialize_chain_and_contract(chain_timestamp);

  let contract_settings = get_view_settings(&chain, contract_address);
  // println!("contract_settings: {:?}", contract_settings);

  assert_eq!(contract_settings.minter, MINTER);
  assert_eq!(contract_settings.mint_start, MINT_START);
  assert_eq!(contract_settings.mint_deadline, MINT_DEADLINE);
  assert_eq!(contract_settings.max_total_supply, MAX_TOTAL_SUPPLY);
}

#[concordium_test]
fn test_mint_should_fail_when_not_minter() {
  let chain_timestamp = MINT_START + 1;
  let (mut chain, contract_address) = initialize_chain_and_contract(chain_timestamp);

  // Mint two tokens to Alice.
  let update_result = chain.contract_update(
    SIGNER,
    USER,
    USER_ADDR,
    Energy::from(10000),
    UpdateContractPayload {
      amount: Amount::zero(),
      receive_name: OwnedReceiveName::new_unchecked("test_nft.mint".to_string()),
      address: contract_address,
      message: OwnedParameter::from_serial(&c_mint_params(2)).expect("Mint params"),
    },
  );
  assert!(update_result.is_err(), "Call didnt fail");
}

#[concordium_test]
fn test_owner_should_be_able_to_set_minter() {
  let chain_timestamp = MINT_START + 1;
  let (mut chain, contract_address) = initialize_chain_and_contract(chain_timestamp);

  let contract_settings = get_view_settings(&chain, contract_address);
  assert_eq!(contract_settings.minter, MINTER);

  let new_minter_params = SetMinter { minter: NEW_MINTER };

  let update_result = mint_to_address(&mut chain, contract_address, c_mint_params(2), None, None);
  assert!(update_result.is_ok(), "Call didnt fail");

  // Change minter
  let update_result = chain.contract_update(
    SIGNER,
    OWNER,
    OWNER_ADDR,
    Energy::from(10000),
    UpdateContractPayload {
      amount: Amount::zero(),
      receive_name: OwnedReceiveName::new_unchecked("test_nft.setMinter".to_string()),
      address: contract_address,
      message: OwnedParameter::from_serial(&new_minter_params).expect("Minter params"),
    },
  );
  assert!(update_result.is_ok(), "Call didnt succeed");

  // Mint with old minter should fail
  let update_result = mint_to_address(&mut chain, contract_address, c_mint_params(42), None, None);
  assert!(update_result.is_err(), "Call didnt fail");

  // Mint with new minter
  let update_result = mint_to_address(
    &mut chain,
    contract_address,
    c_mint_params(42),
    Some(new_minter_params.minter),
    Some(Address::Account(new_minter_params.minter)),
  );
  assert!(update_result.is_ok(), "Call didnt succeed");

  let contract_settings = get_view_settings(&chain, contract_address);
  assert_eq!(contract_settings.minter, new_minter_params.minter);
}

/// Helper function that sets up the contract with two tokens minted to the given recipient
fn mint_to_address(
  chain: &mut Chain,
  contract_address: ContractAddress,
  mint_params: MintParams,
  invoker: Option<AccountAddress>,
  sender: Option<Address>,
) -> Result<ContractInvokeSuccess, ContractInvokeError> {
  let invoker = invoker.unwrap_or(MINTER);
  let sender = sender.unwrap_or(MINTER_ADDR);

  // Mint two tokens to Alice.
  let update_result = chain.contract_update(
    SIGNER,
    invoker,
    sender,
    Energy::from(10000),
    UpdateContractPayload {
      amount: Amount::zero(),
      receive_name: OwnedReceiveName::new_unchecked("test_nft.mint".to_string()),
      address: contract_address,
      message: OwnedParameter::from_serial(&mint_params).expect("Mint params"),
    },
  );

  update_result
}

/// Setup chain and contract.
///
/// Also creates the two accounts, Alice and Bob.
///
/// Alice is the owner of the contract.
fn initialize_chain_and_contract(timestamp: u64) -> (Chain, ContractAddress) {
  let mut chain = Chain::builder()
    .block_time(Timestamp::from_timestamp_millis(timestamp))
    .build()
    .unwrap();

  // Create some accounts accounts on the chain.
  chain.create_account(Account::new(OWNER, ACC_INITIAL_BALANCE));
  chain.create_account(Account::new(MINTER, ACC_INITIAL_BALANCE));
  chain.create_account(Account::new(USER, ACC_INITIAL_BALANCE));
  chain.create_account(Account::new(NEW_MINTER, ACC_INITIAL_BALANCE));

  // Load and deploy the module.
  let module = module_load_v1("nft_test.wasm.v1").expect("Module exists");
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
        init_name: OwnedContractName::new_unchecked("init_test_nft".to_string()),
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

fn get_view_state(chain: &Chain, contract_address: ContractAddress) -> ViewState {
  let invoke = chain
    .contract_invoke(
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.view".to_string()),
        address: contract_address,
        message: OwnedParameter::empty(),
      },
    )
    .expect("Invoke view");

  invoke.parse_return_value().expect("ViewState return value")
}

fn get_view_settings(chain: &Chain, contract_address: ContractAddress) -> ViewSettings {
  let invoke = chain
    .contract_invoke(
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.viewSettings".to_string()),
        address: contract_address,
        message: OwnedParameter::empty(),
      },
    )
    .expect("Invoke view");

  invoke.parse_return_value().expect("ViewState return value")
}

fn c_mint_params(token: u32) -> MintParams {
  MintParams {
    owners: vec![USER_ADDR],
    tokens: vec![TokenIdU32(token)],
    token_uris: vec!["ipfs://test".to_string()],
  }
}

fn get_contract_metadata() -> MetadataUrl {
  MetadataUrl {
    url: "ipfs://contractURI".to_string(),
    hash: None,
  }
}
