use std::cell::RefCell;

use super::*;
use crate as pallet_cf_vaults;
use cf_chains::{
	eth,
	mocks::{MockAggKey, MockEthereum},
	ApiCall, ChainCrypto, ReplayProtectionProvider,
};
use cf_primitives::{BroadcastId, GENESIS_EPOCH};
use cf_traits::{
	impl_mock_callback, impl_mock_chainflip,
	mocks::{
		ceremony_id_provider::MockCeremonyIdProvider,
		eth_replay_protection_provider::MockEthReplayProtectionProvider,
		threshold_signer::MockThresholdSigner,
	},
	AccountRoleRegistry,
};
use frame_support::{
	construct_runtime, parameter_types, traits::UnfilteredDispatchable, StorageHasher,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<MockRuntime>;
type Block = frame_system::mocking::MockBlock<MockRuntime>;

pub type ValidatorId = u64;

thread_local! {
	pub static BAD_VALIDATORS: RefCell<Vec<ValidatorId>> = RefCell::new(vec![]);
	pub static CURRENT_SYSTEM_STATE: RefCell<SystemState> = RefCell::new(SystemState::Normal);

}

construct_runtime!(
	pub enum MockRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		VaultsPallet: pallet_cf_vaults,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

#[derive(Clone, Eq, PartialEq, Copy, Debug)]
pub enum SystemState {
	Normal,
	Maintenance,
}

pub const ETH_DUMMY_SIG: eth::SchnorrVerificationComponents =
	eth::SchnorrVerificationComponents { s: [0xcf; 32], k_times_g_address: [0xcf; 20] };

// do not know how to solve this mock
pub struct MockSystemStateManager;

impl MockSystemStateManager {
	pub fn set_system_state(state: SystemState) {
		CURRENT_SYSTEM_STATE.with(|cell| {
			*cell.borrow_mut() = state;
		});
	}
}

impl SystemStateManager for MockSystemStateManager {
	fn activate_maintenance_mode() {
		Self::set_system_state(SystemState::Maintenance);
	}
}

impl MockSystemStateManager {
	pub fn get_current_system_state() -> SystemState {
		CURRENT_SYSTEM_STATE.with(|cell| *cell.borrow())
	}
}

impl frame_system::Config for MockRuntime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<5>;
}

impl_mock_chainflip!(MockRuntime);
impl_mock_callback!(RuntimeOrigin);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct MockSetAggKeyWithAggKey {
	nonce: <MockEthereum as ChainAbi>::ReplayProtection,
	new_key: <MockEthereum as ChainCrypto>::AggKey,
}

impl SetAggKeyWithAggKey<MockEthereum> for MockSetAggKeyWithAggKey {
	fn new_unsigned(
		old_key: Option<<MockEthereum as ChainCrypto>::AggKey>,
		new_key: <MockEthereum as ChainCrypto>::AggKey,
	) -> Result<Self, ()> {
		old_key.ok_or(())?;
		Ok(Self { nonce: MockEthReplayProtectionProvider::replay_protection(), new_key })
	}
}

impl ApiCall<MockEthereum> for MockSetAggKeyWithAggKey {
	fn threshold_signature_payload(&self) -> <MockEthereum as ChainCrypto>::Payload {
		unimplemented!()
	}

	fn signed(
		self,
		_threshold_signature: &<MockEthereum as ChainCrypto>::ThresholdSignature,
	) -> Self {
		unimplemented!()
	}

	fn chain_encoded(&self) -> Vec<u8> {
		unimplemented!()
	}

	fn is_signed(&self) -> bool {
		unimplemented!()
	}
}

pub struct MockVaultTransitionHandler;
impl VaultTransitionHandler<MockEthereum> for MockVaultTransitionHandler {
	fn on_new_vault() {}
}

pub struct MockBroadcaster;

impl MockBroadcaster {
	pub fn send_broadcast() {
		storage::hashed::put(&<Twox64Concat as StorageHasher>::hash, b"MockBroadcaster", &());
	}

	pub fn broadcast_sent() -> bool {
		storage::hashed::exists(&<Twox64Concat as StorageHasher>::hash, b"MockBroadcaster")
	}
}

impl Broadcaster<MockEthereum> for MockBroadcaster {
	type ApiCall = MockSetAggKeyWithAggKey;
	type Callback = MockCallback;

	fn threshold_sign_and_broadcast(
		_api_call: Self::ApiCall,
	) -> (BroadcastId, ThresholdSignatureRequestId) {
		Self::send_broadcast();
		(1, 2)
	}

	fn threshold_sign_and_broadcast_with_callback(
		_api_call: Self::ApiCall,
		_callback: Self::Callback,
	) -> (BroadcastId, ThresholdSignatureRequestId) {
		unimplemented!()
	}
}

parameter_types! {
	pub const KeygenResponseGracePeriod: u64 = 25;
}

pub type MockOffenceReporter =
	cf_traits::mocks::offence_reporting::MockOffenceReporter<ValidatorId, PalletOffence>;

pub struct MockSlasher;

impl Slashing for MockSlasher {
	type AccountId = ValidatorId;
	type BlockNumber = u64;

	fn slash(_validator_id: &Self::AccountId, _blocks: Self::BlockNumber) {}

	fn slash_balance(_account_id: &Self::AccountId, _amount: sp_runtime::Percent) {}
}

impl pallet_cf_vaults::Config for MockRuntime {
	type RuntimeEvent = RuntimeEvent;
	type Offence = PalletOffence;
	type Chain = MockEthereum;
	type RuntimeCall = RuntimeCall;
	type EnsureThresholdSigned = NeverFailingOriginCheck<Self>;
	type ThresholdSigner = MockThresholdSigner<MockEthereum, RuntimeCall>;
	type OffenceReporter = MockOffenceReporter;
	type SetAggKeyWithAggKey = MockSetAggKeyWithAggKey;
	type VaultTransitionHandler = MockVaultTransitionHandler;
	type CeremonyIdProvider = MockCeremonyIdProvider;
	type WeightInfo = ();
	type Broadcaster = MockBroadcaster;
	type SystemStateManager = MockSystemStateManager;
	type Slasher = MockSlasher;
}

pub const ALICE: <MockRuntime as frame_system::Config>::AccountId = 123u64;
pub const BOB: <MockRuntime as frame_system::Config>::AccountId = 456u64;
pub const CHARLIE: <MockRuntime as frame_system::Config>::AccountId = 789u64;
pub const GENESIS_AGG_PUB_KEY: MockAggKey = MockAggKey(*b"genk");
pub const NEW_AGG_PUB_KEY: MockAggKey = MockAggKey(*b"next");

pub const MOCK_KEYGEN_RESPONSE_TIMEOUT: u64 = 25;

fn test_ext_inner(vault_key: Option<MockAggKey>) -> sp_io::TestExternalities {
	let config = GenesisConfig {
		system: Default::default(),
		vaults_pallet: VaultsPalletConfig {
			vault_key,
			deployment_block: 0,
			keygen_response_timeout: MOCK_KEYGEN_RESPONSE_TIMEOUT,
		},
	};

	let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();

	ext.execute_with(|| {
		System::set_block_number(1);
		let authorities = BTreeSet::from([ALICE, BOB, CHARLIE]);
		for id in &authorities {
			<MockAccountRoleRegistry as AccountRoleRegistry<MockRuntime>>::register_as_validator(
				id,
			)
			.unwrap();
		}
		MockEpochInfo::set_epoch(GENESIS_EPOCH);
		MockEpochInfo::set_epoch_authority_count(
			GENESIS_EPOCH,
			authorities.len() as AuthorityCount,
		);
		MockEpochInfo::set_authorities(authorities);
	});

	ext
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	test_ext_inner(Some(GENESIS_AGG_PUB_KEY))
}

pub(crate) fn new_test_ext_no_key() -> sp_io::TestExternalities {
	test_ext_inner(None)
}
