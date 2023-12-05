use concordium_cis2::{Cis2ClientError, Cis2Error};
use concordium_std::*;

/// The custom errors the contract can produce.
#[derive(Serialize, Debug, PartialEq, Eq, Reject, SchemaType)]
pub enum CustomContractError {
  /// Failed parsing the parameter.
  #[from(ParseError)]
  ParseParams,
  /// Failed logging: Log is full.
  LogFull,
  /// Failed logging: Log is malformed.
  LogMalformed,
  /// Failing to mint new tokens because one of the token IDs already exists
  TokenIdAlreadyExists,
  /// Failed to invoke a contract.
  InvokeContractError,
  /// Minting start unix timestamp is not reached
  MintingNotStarted,
  /// Minting deadline unix timestamp is reached
  MintDeadlineReached,
  /// Max total supply is reached
  MaxTotalSupplyReached,
  /// Tokens, owners or URIs arrays are not of the same length
  ArraysNotSameLength,
  /// Error returned by the CIS2 Client while performing certain operations
  Cis2ClientError,
  /// Not a valid address
  InvalidAddress,
}

/// Wrapping the custom errors in a type with CIS2 errors.
pub type ContractError = Cis2Error<CustomContractError>;

pub type ContractResult<A> = Result<A, ContractError>;

/// Mapping the logging errors to CustomContractError.
impl From<LogError> for CustomContractError {
  fn from(le: LogError) -> Self {
    match le {
      LogError::Full => Self::LogFull,
      LogError::Malformed => Self::LogMalformed,
    }
  }
}

/// Mapping errors related to contract invocations to CustomContractError.
impl<T> From<CallContractError<T>> for CustomContractError {
  fn from(_cce: CallContractError<T>) -> Self {
    Self::InvokeContractError
  }
}

/// Mapping CustomContractError to ContractError
impl From<CustomContractError> for ContractError {
  fn from(c: CustomContractError) -> Self {
    Cis2Error::Custom(c)
  }
}

impl<T> From<Cis2ClientError<T>> for CustomContractError {
  fn from(_: Cis2ClientError<T>) -> Self {
    CustomContractError::Cis2ClientError
  }
}
