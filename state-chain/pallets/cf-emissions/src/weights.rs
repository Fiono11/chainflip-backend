//! Autogenerated weights for pallet_cf_emissions
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-10-28, STEPS: `[20, ]`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Interpreted, CHAIN: None, DB CACHE: 128

// Executed Command:
// ./target/release/state-chain-node
// benchmark
// --extrinsic
// *
// --pallet
// pallet_cf_emissions
// --output
// state-chain/pallets/cf-emissions/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_emissions.
pub trait WeightInfo {
	fn update_backup_validator_emission_inflation(b: u32, ) -> Weight;
	fn update_validator_emission_inflation(b: u32, ) -> Weight;
	fn zero_reward(x: u32, ) -> Weight;
	fn no_rewards_minted() -> Weight;
	fn rewards_minted(x: u32, ) -> Weight;
}

/// Weights for pallet_cf_emissions using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	fn update_backup_validator_emission_inflation(_b: u32, ) -> Weight {
		(66_572_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn update_validator_emission_inflation(_b: u32, ) -> Weight {
		(66_933_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn zero_reward(x: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 6_000
			.saturating_add((773_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn no_rewards_minted() -> Weight {
		(10_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	fn rewards_minted(x: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 12_000
			.saturating_add((1_412_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn update_backup_validator_emission_inflation(_b: u32, ) -> Weight {
		(66_572_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn update_validator_emission_inflation(_b: u32, ) -> Weight {
		(66_933_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn zero_reward(x: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 6_000
			.saturating_add((773_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn no_rewards_minted() -> Weight {
		(10_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
	}
	fn rewards_minted(x: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 12_000
			.saturating_add((1_412_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(RocksDbWeight::get().reads(3 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
}