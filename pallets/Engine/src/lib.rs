#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch, Parameter};
use frame_support::sp_std::fmt::Debug;
use frame_system::ensure_signed;
use sp_runtime::traits::{AtLeast32BitUnsigned, IdentifyAccount, MaybeSerializeDeserialize, Member, Verify};
use primitives::engine::{BalanceState,Log,State,SpotTrade,Commitment,FraudProof,OrderType};

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
    type AssetID: Ord;
}

decl_storage! {
	trait Store for Module<T: Config> as Engine {
	    /// Stores the complete trader balance state
	    Balances get(fn get_traders): map hasher(blake2_128_concat) T::AccountId => State<T::AssetID, T::Balance>;
	    /// Unconfirmed Commitments submitted by the cloud
	    Commitments get(fn get_commitments): map hasher(blake2_128_concat) T::BlockNumber => Commitment<T::AccountId, T::Balance, T::Signature, T::AssetID>;
	    /// Operation Status of Exchange
	    ExchangeStatus: bool = true;
	    /// Valid FraudProofs
	    FraudProofs get(fn get_fraud_proofs): map hasher(blake2_128_concat) T::AccountId => FraudProof<T::AssetID, T::Balance,T::Signature,T::AssetID>;
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

		#[weight = 0]
		pub fn settle_trade(origin) -> dispatch::DispatchResult {
			let cloud_provider = ensure_signed(origin)?;

			Ok(())
		}
	}
}

impl<T: Config> Module<T> {

    // fn verify_signatures(maker: &Order<T::Balance, T::AccountId, T::Hash>,
    //                      taker: &Order<T::Balance, T::AccountId, T::Hash>,
    //                     maker_msg: &T::Hash,
    //                     taker_msg: &T::Hash) -> bool {
    //
    //     // sr25519 always expects a 64 byte signature.
    //     // ensure!(maker.signature.len() == 64 && taker.signature.len() == 64, Error::<T>::InvalidSignature);
    //     let maker_signature: sr25519::Signature = sr25519::Signature::from_slice(&maker.signature).into();
    //
    //     // In Polkadot, the AccountId is always the same as the 32 byte public key.
    //     let maker_account_bytes: [u8; 32] = account_to_bytes(&maker.trader).unwrap();
    //     let maker_public_key = sr25519::Public::from_raw(maker_account_bytes);
    //
    //     let taker_signature: sr25519::Signature = sr25519::Signature::from_slice(&taker.signature).into();
    //
    //     // In Polkadot, the AccountId is always the same as the 32 byte public key.
    //     let taker_account_bytes: [u8; 32] = account_to_bytes(&taker.trader).unwrap();
    //     let taker_public_key = sr25519::Public::from_raw(taker_account_bytes);
    //
    //     taker_signature.verify(&taker_msg.encode()[..], &taker_public_key) && maker_signature.verify(&maker_msg.encode()[..], &maker_public_key)
    //
    //
    // }

}

// This function converts a 32 byte AccountId to its byte-array equivalent form.
// fn account_to_bytes<AccountId>(account: &AccountId) -> Result<[u8; 32], DispatchError>
//     where AccountId: Encode,
// {
//     let account_vec = account.encode();
//     ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
//     let mut bytes = [0u8; 32];
//     bytes.copy_from_slice(&account_vec);
//     Ok(bytes)
// }