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
pub struct SpotTrade<AccountID: Ord + Default, Balance, Signature, AssetID: Ord> {
    trader: AccountID,
    price: Balance,
    amount: Balance,
    order_type: OrderType,
    base_asset: AssetID,
    quote_asset: AssetID,
    nonce: u128,
    signature: Signature,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Log<AccountID: Ord + Default, Balance, Signature, AssetID: Ord> {
    SpotSettlement(SpotTrade<AccountID, Balance, Signature, AssetID>, SpotTrade<AccountID, Balance, Signature, AssetID>, u128),
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
    pub balances: btree_map::BTreeMap<(AccountID, AssetID), BalanceState<Balance>>
}

impl<AssetID: Ord, Balance, AccountID: Ord + Default> Default for State<AssetID, Balance, AccountID> {
    fn default() -> Self {
        State {
            balances: btree_map::BTreeMap::new()
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Commitment<AccountID: Ord + Default, Balance, Signature, AssetID: Ord> {
    pub logs: Vec<Log<AccountID, Balance, Signature, AssetID>>,
    pub final_state: Vec<((AccountID, AssetID), BalanceState<Balance>)>,
}

impl<AccountID: Ord + Default, Balance, Signature, AssetID: Ord> Default for Commitment<AccountID, Balance, Signature, AssetID> {
    fn default() -> Self {
        Commitment {
            logs: vec![],
            final_state: vec![],
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct FraudProof<AccountID: Ord + Default, Balance, Signature, AssetID: Ord> {
    fisherman: AccountID,
    invalid_transitions: Vec<Log<AccountID, Balance, Signature, AssetID>>,
    final_state_should_be: Vec<((AccountID, AssetID), BalanceState<Balance>)>,
}

impl<AccountID: Ord + Default, Balance, Signature, AssetID: Ord> Default for FraudProof<AccountID, Balance, Signature, AssetID> {
    fn default() -> Self {
        FraudProof {
            fisherman: Default::default(),
            invalid_transitions: vec![],
            final_state_should_be: vec![],
        }
    }
}