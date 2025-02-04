
//! Autogenerated weights for pallet_cf_tokenholder_governance
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-11-03, STEPS: `20`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ip-172-31-9-222`, CPU: `Intel(R) Xeon(R) Platinum 8275CL CPU @ 3.00GHz`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./chainflip-node
// benchmark
// pallet
// --pallet
// pallet_cf_tokenholder_governance
// --extrinsic
// *
// --output
// state-chain/pallets/cf-tokenholder-governance/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_cf_tokenholder_governance.
pub trait WeightInfo {
	fn on_initialize_resolve_votes(a: u32, ) -> Weight;
	fn on_initialize_execute_proposal() -> Weight;
	fn submit_proposal() -> Weight;
	fn back_proposal(a: u32, ) -> Weight;
}

/// Weights for pallet_cf_tokenholder_governance using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	/// Storage: `TokenholderGovernance::Proposals` (r:1 w:1)
	/// Proof: `TokenholderGovernance::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::Backers` (r:1 w:1)
	/// Proof: `TokenholderGovernance::Backers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::Account` (r:1000 w:0)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Flip::TotalIssuance` (r:1 w:0)
	/// Proof: `Flip::TotalIssuance` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Flip::OffchainFunds` (r:1 w:0)
	/// Proof: `Flip::OffchainFunds` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `TokenholderGovernance::CommKeyUpdateAwaitingEnactment` (r:1 w:0)
	/// Proof: `TokenholderGovernance::CommKeyUpdateAwaitingEnactment` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::GovKeyUpdateAwaitingEnactment` (r:0 w:1)
	/// Proof: `TokenholderGovernance::GovKeyUpdateAwaitingEnactment` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[10, 1000]`.
	fn on_initialize_resolve_votes(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `644 + a * (118 ±0)`
		//  Estimated: `4118 + a * (2555 ±0)`
		// Minimum execution time: 67_862_000 picoseconds.
		Weight::from_parts(68_101_000, 4118)
			// Standard Error: 18_101
			.saturating_add(Weight::from_parts(4_134_028, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(Weight::from_parts(0, 2555).saturating_mul(a.into()))
	}
	/// Storage: `TokenholderGovernance::Proposals` (r:1 w:0)
	/// Proof: `TokenholderGovernance::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::GovKeyUpdateAwaitingEnactment` (r:1 w:1)
	/// Proof: `TokenholderGovernance::GovKeyUpdateAwaitingEnactment` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::GovKeys` (r:1 w:1)
	/// Proof: `TokenholderGovernance::GovKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumKeyManagerAddress` (r:1 w:0)
	/// Proof: `Environment::EthereumKeyManagerAddress` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumSignatureNonce` (r:1 w:1)
	/// Proof: `Environment::EthereumSignatureNonce` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumChainId` (r:1 w:0)
	/// Proof: `Environment::EthereumChainId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumBroadcaster::BroadcastIdCounter` (r:1 w:1)
	/// Proof: `EthereumBroadcaster::BroadcastIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumChainTracking::CurrentChainState` (r:1 w:0)
	/// Proof: `EthereumChainTracking::CurrentChainState` (`max_values`: Some(1), `max_size`: Some(40), added: 535, mode: `MaxEncodedLen`)
	/// Storage: `EthereumThresholdSigner::ThresholdSignatureRequestIdCounter` (r:1 w:1)
	/// Proof: `EthereumThresholdSigner::ThresholdSignatureRequestIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CurrentKeyEpochAndState` (r:1 w:0)
	/// Proof: `EthereumVault::CurrentKeyEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::Vaults` (r:1 w:0)
	/// Proof: `EthereumVault::Vaults` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::HistoricalAuthorities` (r:1 w:0)
	/// Proof: `Validator::HistoricalAuthorities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Suspensions` (r:4 w:0)
	/// Proof: `Reputation::Suspensions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `EthereumVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::ThresholdSignatureResponseTimeout` (r:1 w:0)
	/// Proof: `EthereumThresholdSigner::ThresholdSignatureResponseTimeout` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::CeremonyRetryQueues` (r:1 w:1)
	/// Proof: `EthereumThresholdSigner::CeremonyRetryQueues` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::CommKeyUpdateAwaitingEnactment` (r:1 w:0)
	/// Proof: `TokenholderGovernance::CommKeyUpdateAwaitingEnactment` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::Signature` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::Signature` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::PendingCeremonies` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::PendingCeremonies` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::RequestCallback` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::RequestCallback` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn on_initialize_execute_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1415`
		//  Estimated: `12305`
		// Minimum execution time: 109_805_000 picoseconds.
		Weight::from_parts(111_555_000, 12305)
			.saturating_add(T::DbWeight::get().reads(20_u64))
			.saturating_add(T::DbWeight::get().writes(10_u64))
	}
	/// Storage: `Flip::Account` (r:1 w:1)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Flip::TotalIssuance` (r:1 w:1)
	/// Proof: `Flip::TotalIssuance` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `TokenholderGovernance::Backers` (r:0 w:1)
	/// Proof: `TokenholderGovernance::Backers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::Proposals` (r:0 w:1)
	/// Proof: `TokenholderGovernance::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn submit_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `479`
		//  Estimated: `3545`
		// Minimum execution time: 26_813_000 picoseconds.
		Weight::from_parts(27_369_000, 3545)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: `TokenholderGovernance::Backers` (r:1 w:1)
	/// Proof: `TokenholderGovernance::Backers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[1, 1000]`.
	fn back_proposal(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `185 + a * (32 ±0)`
		//  Estimated: `3650 + a * (32 ±0)`
		// Minimum execution time: 10_990_000 picoseconds.
		Weight::from_parts(13_089_416, 3650)
			// Standard Error: 559
			.saturating_add(Weight::from_parts(71_329, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(a.into()))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `TokenholderGovernance::Proposals` (r:1 w:1)
	/// Proof: `TokenholderGovernance::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::Backers` (r:1 w:1)
	/// Proof: `TokenholderGovernance::Backers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Flip::Account` (r:1000 w:0)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Flip::TotalIssuance` (r:1 w:0)
	/// Proof: `Flip::TotalIssuance` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Flip::OffchainFunds` (r:1 w:0)
	/// Proof: `Flip::OffchainFunds` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `TokenholderGovernance::CommKeyUpdateAwaitingEnactment` (r:1 w:0)
	/// Proof: `TokenholderGovernance::CommKeyUpdateAwaitingEnactment` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::GovKeyUpdateAwaitingEnactment` (r:0 w:1)
	/// Proof: `TokenholderGovernance::GovKeyUpdateAwaitingEnactment` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[10, 1000]`.
	fn on_initialize_resolve_votes(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `644 + a * (118 ±0)`
		//  Estimated: `4118 + a * (2555 ±0)`
		// Minimum execution time: 67_862_000 picoseconds.
		Weight::from_parts(68_101_000, 4118)
			// Standard Error: 18_101
			.saturating_add(Weight::from_parts(4_134_028, 0).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().reads((1_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
			.saturating_add(Weight::from_parts(0, 2555).saturating_mul(a.into()))
	}
	/// Storage: `TokenholderGovernance::Proposals` (r:1 w:0)
	/// Proof: `TokenholderGovernance::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::GovKeyUpdateAwaitingEnactment` (r:1 w:1)
	/// Proof: `TokenholderGovernance::GovKeyUpdateAwaitingEnactment` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::GovKeys` (r:1 w:1)
	/// Proof: `TokenholderGovernance::GovKeys` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumKeyManagerAddress` (r:1 w:0)
	/// Proof: `Environment::EthereumKeyManagerAddress` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumSignatureNonce` (r:1 w:1)
	/// Proof: `Environment::EthereumSignatureNonce` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Environment::EthereumChainId` (r:1 w:0)
	/// Proof: `Environment::EthereumChainId` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumBroadcaster::BroadcastIdCounter` (r:1 w:1)
	/// Proof: `EthereumBroadcaster::BroadcastIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumChainTracking::CurrentChainState` (r:1 w:0)
	/// Proof: `EthereumChainTracking::CurrentChainState` (`max_values`: Some(1), `max_size`: Some(40), added: 535, mode: `MaxEncodedLen`)
	/// Storage: `EthereumThresholdSigner::ThresholdSignatureRequestIdCounter` (r:1 w:1)
	/// Proof: `EthereumThresholdSigner::ThresholdSignatureRequestIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CurrentKeyEpochAndState` (r:1 w:0)
	/// Proof: `EthereumVault::CurrentKeyEpochAndState` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::Vaults` (r:1 w:0)
	/// Proof: `EthereumVault::Vaults` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Validator::HistoricalAuthorities` (r:1 w:0)
	/// Proof: `Validator::HistoricalAuthorities` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Reputation::Suspensions` (r:4 w:0)
	/// Proof: `Reputation::Suspensions` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumVault::CeremonyIdCounter` (r:1 w:1)
	/// Proof: `EthereumVault::CeremonyIdCounter` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::ThresholdSignatureResponseTimeout` (r:1 w:0)
	/// Proof: `EthereumThresholdSigner::ThresholdSignatureResponseTimeout` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::CeremonyRetryQueues` (r:1 w:1)
	/// Proof: `EthereumThresholdSigner::CeremonyRetryQueues` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::CommKeyUpdateAwaitingEnactment` (r:1 w:0)
	/// Proof: `TokenholderGovernance::CommKeyUpdateAwaitingEnactment` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::Signature` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::Signature` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::PendingCeremonies` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::PendingCeremonies` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EthereumThresholdSigner::RequestCallback` (r:0 w:1)
	/// Proof: `EthereumThresholdSigner::RequestCallback` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn on_initialize_execute_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1415`
		//  Estimated: `12305`
		// Minimum execution time: 109_805_000 picoseconds.
		Weight::from_parts(111_555_000, 12305)
			.saturating_add(RocksDbWeight::get().reads(20_u64))
			.saturating_add(RocksDbWeight::get().writes(10_u64))
	}
	/// Storage: `Flip::Account` (r:1 w:1)
	/// Proof: `Flip::Account` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Flip::TotalIssuance` (r:1 w:1)
	/// Proof: `Flip::TotalIssuance` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `TokenholderGovernance::Backers` (r:0 w:1)
	/// Proof: `TokenholderGovernance::Backers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TokenholderGovernance::Proposals` (r:0 w:1)
	/// Proof: `TokenholderGovernance::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn submit_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `479`
		//  Estimated: `3545`
		// Minimum execution time: 26_813_000 picoseconds.
		Weight::from_parts(27_369_000, 3545)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: `TokenholderGovernance::Backers` (r:1 w:1)
	/// Proof: `TokenholderGovernance::Backers` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[1, 1000]`.
	fn back_proposal(a: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `185 + a * (32 ±0)`
		//  Estimated: `3650 + a * (32 ±0)`
		// Minimum execution time: 10_990_000 picoseconds.
		Weight::from_parts(13_089_416, 3650)
			// Standard Error: 559
			.saturating_add(Weight::from_parts(71_329, 0).saturating_mul(a.into()))
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(a.into()))
	}
}
