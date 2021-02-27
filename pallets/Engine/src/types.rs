use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::collections::btree_map;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum OrderType {
    BidLimit,
    BidMarket,
    AskLimit,
    AskMarket,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TradeStatus<Balance> {
    None,
    PartialFill(Balance),
    Filled,
    Cancelled,
}

impl<Balance> Default for TradeStatus<Balance>{
    fn default() -> Self {
        TradeStatus::None
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Order<Balance, AccountId, AssetID> {
    pub price: Balance,
    pub quantity: Balance,
    pub order_type: OrderType,
    pub trader: AccountId,
    pub nonce: u64,
    pub asset_id: AssetID,
    pub signature: Vec<u8>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AccountData<AssetID: Ord, Balance> {
    pub nonce: u64,
    // TODO: Store nonce in a better data structure
    pub assets: btree_map::BTreeMap<AssetID, Balance>,
}

impl<Balance: Default, AssetID: Ord> Default for AccountData<AssetID, Balance> {
    fn default() -> Self {
        AccountData {
            nonce: 0,
            assets: btree_map::BTreeMap::new(),
        }
    }
}
