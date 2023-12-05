//! Tests for the `ciphers_nft` contract.
use ciphers_nft::cis2::*;
use concordium_cis2::*;
use concordium_smart_contract_testing::*;

pub const OWNER: AccountAddress = AccountAddress([1; 32]);
pub const OWNER_ADDR: Address = Address::Account(OWNER);
pub const MINTER: AccountAddress = AccountAddress([2; 32]);
pub const MINTER_ADDR: Address = Address::Account(MINTER);
pub const USER: AccountAddress = AccountAddress([3; 32]);
pub const USER_ADDR: Address = Address::Account(USER);
pub const USER2: AccountAddress = AccountAddress([4; 32]);
pub const USER2_ADDR: Address = Address::Account(USER2);
pub const USER3: AccountAddress = AccountAddress([5; 32]);
pub const USER3_ADDR: Address = Address::Account(USER3);
pub const NEW_MINTER: AccountAddress = AccountAddress([6; 32]);

/// Token IDs.
pub const TOKEN_0: ContractTokenId = TokenIdU32(2);
pub const TOKEN_1: ContractTokenId = TokenIdU32(42);

/// Initial balance of the accounts.
pub const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(10000);

/// A signer for all the transactions.
pub const SIGNER: Signer = Signer::with_one_key();

pub const NAME: &str = "test nft contract";
pub const SYMBOL: &str = "TST";
pub const MINT_START: u64 = 100;
pub const MINT_DEADLINE: u64 = 1000;
pub const MAX_TOTAL_SUPPLY: u32 = 10;
