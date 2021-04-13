use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok, storage::StorageMap};

#[test]
fn building_a_candidate_list() {
    new_test_ext().execute_with(|| {
        // Pull a list of candidates from cf-staking
    });
}

#[test]
fn have_optional_validators_on_genesis() {
    new_test_ext().execute_with(|| {
        // Add two validators at genesis
        // Confirm we have them from block 1 in the validator set
    });
}

#[test]
fn you_have_to_be_priviledged() {
    new_test_ext().execute_with(|| {
        // Run through the sudo extrinsics to be sure they are what they are
    });
}

#[test]
fn bring_forward_era() {
    new_test_ext().execute_with(|| {
        // Get current next era block number
        // Update next era (block number - 1)
        // Wait (block number - 1) blocks
        // Confirm things have switched
    });
}

#[test]
fn push_back_era() {
    new_test_ext().execute_with(|| {
        // Get current next era block number
        // Update next era (block number + 1)
        // Wait (block number + 1) blocks
        // Confirm we had a switch
    });
}

#[test]
fn limit_validator_set_size() {
    new_test_ext().execute_with(|| {
        // Get current validator size
        // Update (validator size - 1)
        // Force a rotation
        // Confirm we have a (validator - 1) set size
    });
}

#[test]
fn force_rotation() {
    new_test_ext().execute_with(|| {
        // Force rotation
        // Get validator size
        // Check it has rotated with the set validator size
    });
}
