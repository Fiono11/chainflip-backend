// Code mostly taken from here: https://github.com/gautamdhameja/substrate-validator-set
// modifications to it, such as validation (since we're not using sudo to add validators)
// will come later

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod tests;

pub use pallet::*;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;
type Days = u32;
type ValidatorSize = u32;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AuctionStarted(),
        AuctionEnded(),
        EpochChanged(Days, Days, T::AccountId),
        MaximumValidatorsChanged(ValidatorSize, ValidatorSize, T::AccountId)
    }

    #[pallet::error]
    pub enum Error<T> {
        NoValidators,
    }

    // Pallet implements [`Hooks`] trait to define some logic to execute in some context.
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// New validator's session keys should be set in session module before calling this.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub(super) fn set_epoch(
            origin: OriginFor<T>,
            days: Days,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub(super) fn remove_validator(
            origin: OriginFor<T>,
            size: ValidatorSize,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub(super) fn rotate(
            origin: OriginFor<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Ok(().into())
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn epoch_days)]
    pub(super) type EpochDays<T: Config> = StorageValue<_, Days, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn max_validators)]
    pub(super) type MaxValidators<T: Config> = StorageValue<_, ValidatorSize, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub max_validators: ValidatorSize,
        pub epoch_days: Days,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                max_validators: 0,
                epoch_days: 0,
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {}
    }
}

/// Indicates to the session module if the session should be rotated.
/// We set this flag to true when we add/remove a validator.
impl<T: Config> pallet_session::ShouldEndSession<T::BlockNumber> for Pallet<T> {
    fn should_end_session(_now: T::BlockNumber) -> bool {
        false
    }
}

/// Provides the new set of validators to the session module when session is being rotated.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
    fn new_session(_new_index: u32) -> Option<Vec<T::AccountId>> {
        None
    }

    fn end_session(_end_index: u32) {}

    fn start_session(_start_index: u32) {}
}

impl<T: Config> frame_support::traits::EstimateNextSessionRotation<T::BlockNumber> for Pallet<T> {
    fn estimate_next_session_rotation(_now: T::BlockNumber) -> Option<T::BlockNumber> {
        None
    }

    // The validity of this weight depends on the implementation of `estimate_next_session_rotation`
    fn weight(_now: T::BlockNumber) -> u64 {
        0
    }
}

/// Implementation of Convert trait for mapping ValidatorId with AccountId.
/// This is mainly used to map stash and controller keys.
/// In this module, for simplicity, we just return the same AccountId.
pub struct ValidatorOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<T::AccountId>> for ValidatorOf<T> {
    fn convert(account: T::AccountId) -> Option<T::AccountId> {
        Some(account)
    }
}

impl<T: Config> Pallet<T> {
    pub fn get_validators() -> Result<Vec<T::AccountId>, &'static str> {
        Err("No validators found")
    }

    pub fn is_validator(account_id: &T::AccountId) -> bool {
        return false;
    }
}
