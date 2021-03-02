use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;
use sp_std::collections::btree_map;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum OrderType {
    BidLimit,
    AskLimit,
    BidMarket,
    AskMarket,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum VerificationErrors{
    OrderAmountFailure,
    OrderPriceFailure,
    InvalidOrderCombination,
    OrderParamCheckFailed,
    OrderSignatureFailure,
    InvalidFinalState,
    InvalidFee

}


#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SpotTrade<AccountID: Ord + Default, Balance, AssetID: Ord> {
    pub trader: AccountID,
    pub price: Balance,
    pub amount: Balance,
    pub order_type: OrderType,
    pub base_asset: AssetID,
    pub quote_asset: AssetID,
    pub nonce: u128,
    pub signature: Vec<u8>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Log<AccountID: Ord + Default, Balance, AssetID: Ord> {
    /// SpotSettlement(maker,taker, fee as fraction, nonce)
    SpotSettlement(SpotTrade<AccountID, Balance, AssetID>, SpotTrade<AccountID, Balance, AssetID>, Balance, u128),
    Withdrawal(AccountID, AssetID, Balance, u128),
    Deposit(AccountID, AssetID, Balance, u128),
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BalanceState<Balance> {
    pub unconfirmed_balance: Balance,
    pub confirmed_balance: Balance,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct State<AssetID: Ord, Balance, AccountID: Ord + Default> {
    pub balances: btree_map::BTreeMap<(AccountID, AssetID), BalanceState<Balance>>,
    pub nonce: u128,
    pub nonces: btree_map::BTreeMap<AccountID,u128>
}

impl<AssetID: Ord, Balance, AccountID: Ord + Default> Default for State<AssetID, Balance, AccountID> {
    fn default() -> Self {
        State {
            balances: btree_map::BTreeMap::new(),
            nonce: 0,
            nonces: btree_map::BTreeMap::new()
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Commitment<AccountID: Ord + Default, Balance, AssetID: Ord> {
    pub logs: Vec<Log<AccountID, Balance, AssetID>>,
    pub final_state: Vec<((AccountID, AssetID), BalanceState<Balance>)>,
    pub nonce: u128
}

impl<AccountID: Ord + Default, Balance, AssetID: Ord> Default for Commitment<AccountID, Balance, AssetID> {
    fn default() -> Self {
        Commitment {
            logs: vec![],
            final_state: vec![],
            nonce: 0
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct FraudProof<AccountID: Ord + Default, Balance, AssetID: Ord> {
    fisherman: AccountID,
    invalid_transitions: Vec<Log<AccountID, Balance, AssetID>>,
    final_state_should_be: Vec<((AccountID, AssetID), BalanceState<Balance>)>,
}

impl<AccountID: Ord + Default, Balance, AssetID: Ord> Default for FraudProof<AccountID, Balance, AssetID> {
    fn default() -> Self {
        FraudProof {
            fisherman: Default::default(),
            invalid_transitions: vec![],
            final_state_should_be: vec![],
        }
    }
}