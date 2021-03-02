#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_error, decl_event, decl_module,
                    decl_storage, dispatch, Parameter, ensure,
                    sp_std::fmt::Debug,
                    traits::Get,
                    weights::Weight};
use frame_system::ensure_signed;
use sp_core::Hasher;
use sp_runtime::{DispatchError, ModuleId};
use sp_runtime::app_crypto::sp_core::sr25519;
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, IdentifyAccount, MaybeSerializeDeserialize, Member, Saturating, Verify, Zero};
use sp_std::collections::btree_map;

use polkadex_primitives::engine::{BalanceState, Commitment, FraudProof, Log, OrderType, SpotTrade, State, VerificationErrors};

#[cfg(test)]
mod mock;
mod benchmarking;
mod tests;


/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    /// Balance Type
    type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + Debug + MaybeSerializeDeserialize;
    /// Public Key of the trader
    type Public: IdentifyAccount<AccountId=Self::AccountId>;
    /// Signature provided by the trade
    type Signature: Verify<Signer=Self::Public> + Member + Decode + Encode;
    /// Asset ID
    type AssetID: Ord + Encode + Decode + Clone + Debug;
    /// Dispute Period
    type DisputePeriod: Get<<Self as frame_system::Config>::BlockNumber>;
    /// Maximum Exchange Fee
    type MaxTradingFee: Get<<Self as Config>::Balance>;
}

decl_storage! {
	trait Store for Module<T: Config> as Engine {
	    /// Stores the complete trader balance state
	    Balances get(fn get_traders): State<T::AssetID, T::Balance, T::AccountId>;
	    /// Unconfirmed Commitments submitted by the cloud
	    Commitments get(fn get_commitments): map hasher(blake2_128_concat) T::BlockNumber => Commitment<T::AccountId, T::Balance, T::AssetID>;
	    /// Operation Status of Exchange
	    ExchangeStatus: bool = true;
	    /// Valid FraudProofs
	    FraudProofs get(fn get_fraud_proofs): map hasher(blake2_128_concat) T::BlockNumber => FraudProof<T::AccountId, T::Balance, T::AssetID>;
	    /// Registered Providers
	    Providers get(fn get_providers): map hasher(blake2_128_concat) T::AccountId => bool;
	}
}


decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// The caller is not registered on the blockchain
		CallerNotARegisteredProvider,
		/// Trader Signature mismatch
		TraderSignatureMismatch,
		/// Outdated Trade
		NonceAlreadyUsed,
		/// OrderType Given For Maker and Taker is invalid
		InvalidOrderTypeCombination,
		/// Signature provided is invalid
		InvalidSignature,

	}
}


decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		fn on_initialize(now: T::BlockNumber) -> Weight{
		    Self::finalize_commitment(now)
		}

		#[weight = 0]
		pub fn submit_commitment(origin, commitment: Commitment<T::AccountId, T::Balance, T::AssetID>) -> dispatch::DispatchResult {
			let provider = ensure_signed(origin)?;
			if <Providers<T>>::contains_key(provider) && !<Commitments<T>>::contains_key(<frame_system::Module<T>>::block_number()){ //
			    <Commitments<T>>::insert(<frame_system::Module<T>>::block_number(), commitment);
			}
			Ok(())
		}

		fn offchain_worker(now: T::BlockNumber){
		    Self::execute_offchain_worker(now);
		}
	}
}

impl<T: Config> Module<T> {
    /// Stores fees collected from trades
    pub fn get_exchange_account() -> T::AccountId {
        ModuleId(*b"exchange").into_account()
    }

    fn finalize_commitment(now: T::BlockNumber) -> Weight {
        if now - T::DisputePeriod::get() >= 0.into() && !<FraudProofs<T>>::contains_key(now - T::DisputePeriod::get()) {
            let commitment: Commitment<T::AccountId, T::Balance, T::AssetID> = <Commitments<T>>::take(now - T::DisputePeriod::get());
            let mut prev_state: State<T::AssetID, T::Balance, T::AccountId> = <Balances<T>>::get();
            for state in 0..commitment.final_state.len() { // FIXME: Bound this loop.
                prev_state.balances.insert(commitment.final_state[state].0.clone(), commitment.final_state[state].1.clone());
            }
            prev_state.nonce = commitment.nonce;
            <Balances<T>>::put(prev_state);
            return Weight::zero(); // FIXME: Implement proper weight based on number of loops
        }
        Weight::zero()
    }

    fn execute_offchain_worker(now: T::BlockNumber) {
        let target_block: T::BlockNumber = now - T::DisputePeriod::get();
        if target_block >= 0.into() && <Commitments<T>>::contains_key(target_block) {
            let target_commitment: Commitment<T::AccountId, T::Balance, T::AssetID> = <Commitments<T>>::get(target_block);
            let mut prev_state: State<T::AssetID, T::Balance, T::AccountId> = <Balances<T>>::get();
            for log in target_commitment.logs {
                match log {
                    Log::SpotSettlement(maker, taker, fee, nonce) => {
                        // FIXME: Verify the nonces of each trade
                        let maker_msg: T::Hash = (maker.price, maker.amount, maker.order_type, maker.base_asset, maker.quote_asset, maker.nonce).using_encoded(<T as frame_system::Config>::Hashing::hash);
                        let taker_msg: T::Hash = (taker.price, taker.amount, taker.order_type, taker.base_asset, taker.quote_asset, taker.nonce).using_encoded(<T as frame_system::Config>::Hashing::hash);
                        if Self::verify_signatures(&maker, &taker, &maker_msg, &taker_msg) && Self::verify_nonce(nonce, prev_state.nonce) {
                            Self::verify_trades_and_update(&maker, &taker, &mut prev_state.balances, fee);
                        } else {
                            Self::raise_dispute(VerificationErrors::OrderSignatureFailure);
                        }
                    }
                    Log::Deposit(trader, assetID, amount, nonce) => {
                        if Self::verify_nonce(nonce, prev_state.nonce) {
                            // TODO: Update State

                            prev_state.nonce = nonce;
                        }
                    }
                    Log::Withdrawal(trader, assetID, amount, nonce) => {
                        if Self::verify_nonce(nonce, prev_state.nonce) {
                            // TODO: Update State

                            prev_state.nonce = nonce;
                        }
                    }
                }
            }
            if !Self::verify_state(&prev_state, &target_commitment.final_state) {
                Self::raise_dispute(VerificationErrors::InvalidFinalState);
            }
        }
    }

    /// BTC/USD: BTC is the base asset, USD is the quote asset.
    /// Price is defined in terms of quote asset
    /// Amount is defined in terms of base asset
    fn verify_trades_and_update(maker: &SpotTrade<T::AccountId, T::Balance, T::AssetID>,
                                taker: &SpotTrade<T::AccountId, T::Balance, T::AssetID>,
                                state: &mut btree_map::BTreeMap<(T::AccountId, T::AssetID), BalanceState<T::Balance>>,
                                fee: T::Balance) {
        if fee > T::MaxTradingFee::get() {
            Self::raise_dispute(VerificationErrors::InvalidFee);
            return;
        }

        match (maker.order_type, taker.order_type) {
            /// These are the possible combinations for a given maker and taker pair
            (OrderType::BidLimit, OrderType::AskLimit) => {
                if taker.price <= maker.price && taker.amount <= maker.amount {
                    // Calculate Quote asset
                    // Calculate Base asset
                    // Calculate Fees
                    // Maker is buyer and Taker is seller
                    let trade_amt: T::Balance = taker.price * taker.amount; // Taken from maker
                    let fees_collected_quote: T::Balance = trade_amt * fee;  // Given to exchange
                    let taker_quote_amt: T::Balance = trade_amt - fees_collected_quote; // Given to taker
                    let maker_base_amt: T::Balance = taker.amount; // Given to maker and Taken from taker.


                    // Read the balances
                    // Maker Balances
                    let mut maker_quote: T::Balance = state.get_mut(&(maker.trader, maker.quote_asset)).unwrap().confirmed_balance;
                    let mut maker_base: T::Balance = state.get_mut(&(maker.trader, maker.base_asset)).unwrap().confirmed_balance;
                    // Taker Balances
                    let mut taker_quote: T::Balance = state.get_mut(&(taker.trader, taker.quote_asset)).unwrap().confirmed_balance;
                    let mut taker_base: T::Balance = state.get_mut(&(taker.trader, taker.base_asset)).unwrap().confirmed_balance;
                    // Exchange Balances
                    let mut exchange_balance: T::Balance = state.get_mut(&(Self::get_exchange_account(), taker.quote_asset)).unwrap().confirmed_balance;

                    // Update the balances
                    maker_quote = maker_quote - trade_amt;
                    taker_base = taker_base - maker_base_amt;

                    maker_base = maker_base + maker_base_amt;
                    taker_quote = taker_quote + taker_quote_amt;
                    exchange_balance = exchange_balance + fees_collected_quote;

                } else {
                    Self::raise_dispute(VerificationErrors::OrderParamCheckFailed);
                }
            },
            (OrderType::AskLimit, OrderType::BidLimit) => {
                if maker.price <= taker.price && taker.amount <= maker.amount {
                    // TODO: Calculate Quote asset
                    // TODO: Calculate Base asset
                    // TODO: Calculate Fees
                    // TODO: Transfer amounts
                } else {
                    Self::raise_dispute(VerificationErrors::OrderParamCheckFailed);
                }
            },
            (OrderType::BidLimit, OrderType::AskMarket) => {
                if taker.amount <= maker.amount {
                    // TODO: Calculate Quote asset
                    // TODO: Calculate Base asset
                    // TODO: Calculate Fees
                    // TODO: Transfer amounts
                } else {
                    Self::raise_dispute(VerificationErrors::OrderAmountFailure);
                }
            },
            (OrderType::AskLimit, OrderType::BidMarket) => {
                if taker.amount <= maker.amount {
                    // TODO: Calculate Quote asset
                    // TODO: Calculate Base asset
                    // TODO: Calculate Fees
                    // TODO: Transfer amounts
                } else {
                    Self::raise_dispute(VerificationErrors::OrderAmountFailure);
                }
            },
            _ => {
                Self::raise_dispute(VerificationErrors::InvalidOrderCombination);
            }, /// This shouldn't execute ever.
        };
    }

    fn raise_dispute(error: VerificationErrors) {
        unimplemented!();
    }

    fn verify_state(computed_state: &State<T::AssetID, T::Balance, T::AccountId>,
                    given_state: &Vec<((T::AccountId, T::AssetID), BalanceState<T::Balance>)>) -> bool {
        unimplemented!();
    }

    fn verify_signatures(maker: &SpotTrade<T::AccountId, T::Balance, T::AssetID>,
                         taker: &SpotTrade<T::AccountId, T::Balance, T::AssetID>,
                         maker_msg: &T::Hash,
                         taker_msg: &T::Hash) -> bool {
        // sr25519 always expects a 64 byte signature.
        // ensure!(maker.signature.len() == 64 && taker.signature.len() == 64, Error::<T>::InvalidSignature);
        let maker_signature: sr25519::Signature = sr25519::Signature::from_slice(&maker.signature).into();

        // In Polkadot, the AccountId is always the same as the 32 byte public key.
        let maker_account_bytes: [u8; 32] = account_to_bytes(&maker.trader).unwrap();
        let maker_public_key = sr25519::Public::from_raw(maker_account_bytes);

        let taker_signature: sr25519::Signature = sr25519::Signature::from_slice(&taker.signature).into();

        // In Polkadot, the AccountId is always the same as the 32 byte public key.
        let taker_account_bytes: [u8; 32] = account_to_bytes(&taker.trader).unwrap();
        let taker_public_key = sr25519::Public::from_raw(taker_account_bytes);

        taker_signature.verify(&taker_msg.encode()[..], &taker_public_key) && maker_signature.verify(&maker_msg.encode()[..], &maker_public_key)
    }

    fn verify_nonce(nonce: u128, prev_nonce: u128) -> bool {
        nonce == prev_nonce + 1
    }
}

// This function converts a 32 byte AccountId to its byte-array equivalent form.
fn account_to_bytes<AccountId>(account: &AccountId) -> Result<[u8; 32], DispatchError>
    where AccountId: Encode,
{
    let account_vec = account.encode();
    ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&account_vec);
    Ok(bytes)
}