use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{RuntimeDebug};
use sp_std::collections::btree_map;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum OrderType {
    BidLimit,
    AskLimit,
    BidMarket,
    AskMarket
}


#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SpotTrade<AccountID, Balance, Signature, AssetID: Ord> {
    trader: AccountID,
    price: Balance,
    amount: Balance,
    order_type: OrderType,
    base_asset: AssetID,
    quote_asset: AssetID,
    nonce: u128,
    signature: Signature
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Log<AccountID, Balance, Signature, AssetID: Ord> {
    SpotSettlement(SpotTrade<AccountID, Balance, Signature, AssetID>,SpotTrade<AccountID, Balance, Signature, AssetID>,u128),
    Withdrawal(AccountID,AssetID,Balance,u128),
    Deposit(AccountID,AssetID,Balance,u128)
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BalanceState<Balance> {
    unconfirmed_balance: Balance,
    confirmed_balance: Balance
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct State<AssetID: Ord, Balance> {
    balances: btree_map::BTreeMap<AssetID,BalanceState<Balance>>
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Commitment<AccountID, Balance, Signature, AssetID: Ord> {
    logs: Vec<Log<AccountID, Balance, Signature,AssetID>>,
    final_state: Vec<(AccountID,State<AssetID,Balance>)>
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct FraudProof<AccountID, Balance, Signature, AssetID: Ord> {
    invalid_transitions: Vec<Log<AccountID, Balance, Signature, AssetID>>,
    final_state_should_be: Vec<(AccountID,State<AssetID,Balance>)>,
}