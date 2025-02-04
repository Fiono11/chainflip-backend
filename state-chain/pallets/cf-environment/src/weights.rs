
//! Autogenerated weights for pallet_cf_environment
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-12-13, STEPS: `20`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Maxims-MacBook-Pro-2.local`, CPU: `<UNKNOWN>`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./target/release/chainflip-node
// benchmark
// pallet
// --extrinsic
// *
// --pallet
// pallet_cf_environment
// --output
// state-chain/pallets/cf-environment/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=20
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_cf_environment.
pub trait WeightInfo {
	fn update_safe_mode() -> Weight;
	fn update_consolidation_parameters() -> Weight;
}

/// Weights for pallet_cf_environment using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	/// Storage: `Environment::RuntimeSafeMode` (r:0 w:1)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_safe_mode() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_000_000 picoseconds.
		Weight::from_parts(8_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `Environment::ConsolidationParameters` (r:0 w:1)
	/// Proof: `Environment::ConsolidationParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_consolidation_parameters() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_000_000 picoseconds.
		Weight::from_parts(8_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `Environment::RuntimeSafeMode` (r:0 w:1)
	/// Proof: `Environment::RuntimeSafeMode` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_safe_mode() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_000_000 picoseconds.
		Weight::from_parts(8_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Environment::ConsolidationParameters` (r:0 w:1)
	/// Proof: `Environment::ConsolidationParameters` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	fn update_consolidation_parameters() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_000_000 picoseconds.
		Weight::from_parts(8_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
