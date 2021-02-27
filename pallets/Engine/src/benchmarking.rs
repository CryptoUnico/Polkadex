#![cfg(feature = "runtime-benchmarks")]
use frame_support::traits::Vec;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::ensure;
use frame_system::{EventRecord, RawOrigin};
use sp_std::collections::btree_map;
use sp_runtime::{traits::AccountIdConversion, AccountId32};
use crate::Module as Identity;
use super::*;
use crate::types::OrderType;
use sp_io::crypto;
use sp_core::{H256};
use sp_std::boxed::Box;
const SEED: u32 = 0;

benchmarks! {

	settle_trade {

	}: _(RawOrigin::Signed(caller), maker_order, taker_order)
}

#[cfg(test)]
mod tests {

    use crate::mock::{new_test_ext, Test};

    use super::*;
    use crate::{Error, mock::*};
    use frame_support::{assert_ok, assert_noop};

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_settle_trade::<Test>());
        });
    }
}
