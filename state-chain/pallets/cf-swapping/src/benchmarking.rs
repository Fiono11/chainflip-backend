//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use cf_primitives::*;
use cf_traits::AccountRoleRegistry;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

fn generate_swaps<T: Config>(amount: u32, from: Asset, to: Asset) -> Vec<Swap<T::AccountId>> {
	let mut swaps: Vec<Swap<T::AccountId>> = vec![];
	for _i in 1..amount {
		swaps.push(Swap {
			from,
			to,
			amount: 3,
			egress_address: ForeignChainAddress::Eth(Default::default()),
			relayer_id: whitelisted_caller(),
			relayer_commission_bps: 4,
		});
	}
	swaps
}

benchmarks! {
	register_swap_intent {
		let caller: T::AccountId = whitelisted_caller();
		T::AccountRoleRegistry::register_account(caller.clone(), AccountRole::Relayer);
	}: _(
		RawOrigin::Signed(caller.clone()),
		Asset::Eth,
		Asset::Usdc,
		ForeignChainAddress::Eth(Default::default()),
		0
	)
	on_idle {}: {
		Pallet::<T>::on_idle(T::BlockNumber::from(1u32), 1);
	}
	execute_group_of_swaps {
		// Generate swaps
		let a in 1..150;
		let swaps = generate_swaps::<T>(a, Asset::Eth, Asset::Flip);
	} : {
		Pallet::<T>::execute_group_of_swaps(swaps, Asset::Eth, Asset::Flip);
	}
}