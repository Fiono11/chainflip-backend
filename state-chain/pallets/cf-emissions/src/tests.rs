use crate::{
	mock::*, BlockEmissions, LastSupplyUpdateBlock, Pallet, SUPPLY_UPDATE_INTERVAL_DEFAULT,
};
use cf_traits::{mocks::system_state_info::MockSystemStateInfo, Issuance, RewardsDistribution};
use frame_support::traits::{Imbalance, OnInitialize};
use pallet_cf_flip::{FlipIssuance, Pallet as Flip};

type Emissions = Pallet<Test>;

#[test]
fn test_should_mint() {
	// If supply_update_interval is zero, we mint on every block.
	assert!(Emissions::should_update_supply(0, 0));
	assert!(Emissions::should_update_supply(1, 0));
	// If not enough blocks have elapsed we don't broadcast supply update.
	assert!(!Emissions::should_update_supply(0, 1));
	// If we are at or above the supply_update_interval, we broadcast supply update.
	assert!(Emissions::should_update_supply(1, 1));
	assert!(Emissions::should_update_supply(2, 1));
}

#[test]
fn test_should_mint_at() {
	new_test_ext(vec![], None).execute_with(|| {
		// It has been `SUPPLY_UPDATE_INTERVAL` blocks since the last broadcast.
		assert!(Emissions::should_update_supply_at(SUPPLY_UPDATE_INTERVAL_DEFAULT));
		// It hasn't yet been `SUPPLY_UPDATE_INTERVAL` blocks since the last broadcast.
		assert!(!Emissions::should_update_supply_at(SUPPLY_UPDATE_INTERVAL_DEFAULT - 1));
		// It has been more than `SUPPLY_UPDATE_INTERVAL` blocks since the last broadcast.
		assert!(Emissions::should_update_supply_at(SUPPLY_UPDATE_INTERVAL_DEFAULT + 1));
		// We have literally *just* broadcasted.
		assert!(!Emissions::should_update_supply_at(0));
	});
}

#[cfg(test)]
mod test_block_rewards {
	use super::*;

	fn test_with(emissions_per_block: u128) {
		new_test_ext(vec![1, 2], Some(1000)).execute_with(|| {
			Emissions::update_authority_block_emission(emissions_per_block);

			let before = Flip::<Test>::total_issuance();
			Emissions::mint_rewards_for_block();
			let after = Flip::<Test>::total_issuance();

			assert_eq!(before + emissions_per_block, after);
		});
	}

	#[test]
	fn test_zero_block() {
		test_with(1);
	}

	#[test]
	fn test_zero_emissions_rate() {
		test_with(0);
	}

	#[test]
	fn test_non_zero_rate() {
		test_with(10);
	}
}

#[test]
fn test_duplicate_emission_should_be_noop() {
	const EMISSION_RATE: u128 = 10;

	new_test_ext(vec![1, 2], None).execute_with(|| {
		//const BLOCK_NUMBER: u64 = 5;

		Emissions::update_authority_block_emission(EMISSION_RATE);

		let before = Flip::<Test>::total_issuance();
		Emissions::mint_rewards_for_block();
		let after = Flip::<Test>::total_issuance();

		assert_eq!(before + EMISSION_RATE, after);

		// Minting again at the same block should have no effect.
		let before = after;
		Emissions::mint_rewards_for_block();
		let after = Flip::<Test>::total_issuance();

		assert_eq!(before + EMISSION_RATE, after);
	});
}

#[test]
fn should_calculate_block_emissions() {
	new_test_ext(vec![1, 2], None).execute_with(|| {
		// Block emissions are calculated at genesis.
		assert!(Emissions::current_authority_emission_per_block() > 0);
		assert!(Emissions::backup_node_emission_per_block() > 0);
	});
}

#[test]
fn should_mint_but_not_broadcast() {
	new_test_ext(vec![1, 2], None).execute_with(|| {
		let prev_supply_update_block = LastSupplyUpdateBlock::<Test>::get();
		Emissions::mint_rewards_for_block();
		assert_eq!(prev_supply_update_block, LastSupplyUpdateBlock::<Test>::get());
	});
}

#[test]
fn test_reward_distribution() {
	new_test_ext(vec![1, 2], None).execute_with(|| {
		let before = Flip::<Test>::total_issuance();
		let reward = FlipIssuance::mint(1_000);
		assert!(reward.peek() > 0);
		<MockRewardsDistribution as RewardsDistribution>::distribute(reward);
		let after = Flip::<Test>::total_issuance();
		assert!(after > before, "Expected {:?} > {:?}", after, before);
	});
}

#[test]
fn should_mint_and_initiate_broadcast() {
	new_test_ext(vec![1, 2], None).execute_with(|| {
		let before = Flip::<Test>::total_issuance();
		assert!(MockBroadcast::get_called().is_none());
		<Emissions as OnInitialize<_>>::on_initialize(SUPPLY_UPDATE_INTERVAL_DEFAULT);
		let after = Flip::<Test>::total_issuance();
		assert!(after > before, "Expected {:?} > {:?}", after, before);
		assert_eq!(
			MockBroadcast::get_called().unwrap().new_total_supply,
			Flip::<Test>::total_issuance()
		);
	});
}

#[test]
fn no_update_of_update_total_supply_during_maintanance() {
	new_test_ext(vec![1, 2], None).execute_with(|| {
		// Activate maintenance mode
		MockSystemStateInfo::set_maintenance(true);
		// Try send a broadcast to update the total supply
		<Emissions as OnInitialize<_>>::on_initialize(SUPPLY_UPDATE_INTERVAL_DEFAULT);
		// Expect nothing to be sent
		assert!(MockBroadcast::get_called().is_none());
		// Deactivate maintenance mode
		MockSystemStateInfo::set_maintenance(false);
		// Try send a broadcast to update the total supply
		<Emissions as OnInitialize<_>>::on_initialize(SUPPLY_UPDATE_INTERVAL_DEFAULT * 2);
		// Expect the broadcast to be sendt
		assert_eq!(
			MockBroadcast::get_called().unwrap().new_total_supply,
			Flip::<Test>::total_issuance()
		);
	});
}
