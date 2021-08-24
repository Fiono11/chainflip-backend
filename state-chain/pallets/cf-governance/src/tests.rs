use crate::{mock::*, ActiveProposals, Error, ExpirySpan, Members, NumberOfProposals};
use cf_traits::mocks::time_source;
use frame_support::{assert_noop, assert_ok, traits::OnInitialize};
use std::time::Duration;

use crate as pallet_cf_governance;

fn mock_extrinsic() -> Box<Call> {
	let call = Box::new(Call::Governance(
		pallet_cf_governance::Call::<Test>::new_membership_set(vec![EVE, PETER, MAX]),
	));
	call
}

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	<Governance as OnInitialize<u64>>::on_initialize(System::block_number());
}

fn last_event() -> crate::mock::Event {
	frame_system::Pallet::<Test>::events()
		.pop()
		.expect("Event expected")
		.event
}

#[test]
fn genesis_config() {
	new_test_ext().execute_with(|| {
		let genesis_members = Members::<Test>::get();
		assert!(genesis_members.contains(&ALICE));
		assert!(genesis_members.contains(&BOB));
		assert!(genesis_members.contains(&CHARLES));
		let expiry_span = ExpirySpan::<Test>::get();
		assert_eq!(expiry_span, 50);
	});
}

#[test]
fn governance_restriction() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Governance::new_membership_set(Origin::signed(ALICE), vec![EVE, PETER, MAX]),
			frame_support::error::BadOrigin
		);
	});
}

#[test]
fn not_a_member() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Governance::propose_governance_extrinsic(Origin::signed(EVE), mock_extrinsic()),
			<Error<Test>>::NotMember
		);
	});
}

#[test]
fn propose_a_governance_extrinsic_and_expect_execution() {
	new_test_ext().execute_with(|| {
		assert_ok!(Governance::propose_governance_extrinsic(
			Origin::signed(ALICE),
			mock_extrinsic()
		));
		assert_eq!(
			last_event(),
			crate::mock::Event::pallet_cf_governance(crate::Event::Proposed(1)),
		);
		assert_ok!(Governance::approve(Origin::signed(BOB), 1));
		assert_ok!(Governance::approve(Origin::signed(CHARLES), 1));
		assert_eq!(
			last_event(),
			crate::mock::Event::pallet_cf_governance(crate::Event::Executed(1)),
		);
		let genesis_members = Members::<Test>::get();
		assert!(genesis_members.contains(&EVE));
		assert!(genesis_members.contains(&PETER));
		assert!(genesis_members.contains(&MAX));
		assert_eq!(
			last_event(),
			crate::mock::Event::pallet_cf_governance(crate::Event::Executed(1)),
		);
	});
}

#[test]
fn proposal_not_found() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Governance::approve(Origin::signed(ALICE), 200),
			<Error<Test>>::NotFound
		);
	});
}

#[test]
fn propose_a_governance_extrinsic_and_expect_it_to_expire() {
	new_test_ext().execute_with(|| {
		const START_TIME: Duration = Duration::from_secs(10);
		const END_TIME: Duration = Duration::from_secs(7300);
		time_source::Mock::reset_to(START_TIME);
		next_block();
		assert_ok!(Governance::propose_governance_extrinsic(
			Origin::signed(ALICE),
			mock_extrinsic()
		));
		next_block();
		time_source::Mock::reset_to(END_TIME);
		next_block();
		assert_eq!(
			last_event(),
			crate::mock::Event::pallet_cf_governance(crate::Event::Expired(1)),
		);
		assert_eq!(ActiveProposals::<Test>::get().len(), 0);
	});
}

#[test]
fn several_open_proposals() {
	new_test_ext().execute_with(|| {
		assert_ok!(Governance::propose_governance_extrinsic(
			Origin::signed(ALICE),
			mock_extrinsic()
		));
		assert_eq!(
			last_event(),
			crate::mock::Event::pallet_cf_governance(crate::Event::Proposed(1)),
		);
		assert_ok!(Governance::propose_governance_extrinsic(
			Origin::signed(BOB),
			mock_extrinsic()
		));
		assert_eq!(
			last_event(),
			crate::mock::Event::pallet_cf_governance(crate::Event::Proposed(2)),
		);
		assert_eq!(NumberOfProposals::<Test>::get(), 2);
	});
}

#[test]
fn sudo_extrinsic() {
	new_test_ext().execute_with(|| {
		// Define a sudo call
		let sudo_call = Box::new(Call::System(
			frame_system::Call::<Test>::set_code_without_checks(vec![1, 2, 3, 4]),
		));
		// Wrap the sudo call as governance extrinsic
		let governance_extrinsic = Box::new(Call::Governance(
			pallet_cf_governance::Call::<Test>::call_as_sudo(sudo_call),
		));
		// Propose the governance extrinsic
		assert_ok!(Governance::propose_governance_extrinsic(
			Origin::signed(ALICE),
			governance_extrinsic
		));
		assert_eq!(
			last_event(),
			crate::mock::Event::pallet_cf_governance(crate::Event::Proposed(1)),
		);
		// Do the two necessary approvals
		assert_ok!(Governance::approve(Origin::signed(BOB), 1));
		assert_ok!(Governance::approve(Origin::signed(CHARLES), 1));
		// Expect the sudo extrinsic to be executed successfully
		assert_eq!(
			last_event(),
			crate::mock::Event::pallet_cf_governance(crate::Event::Executed(1)),
		);
	});
}
