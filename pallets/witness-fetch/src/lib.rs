#![cfg_attr(not(feature = "std"), no_std)]

use frame_support as support;
use frame_system as system;

use rstd::prelude::*;

use sp_runtime::{
    offchain::{http, storage::StorageValueRef},
    traits::IdentifyAccount,
    transaction_validity::{InvalidTransaction, ValidTransaction},
    DispatchResult, KeyTypeId,
};

use support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    unsigned::{TransactionSource, TransactionValidity},
};

use system::{
    ensure_none,
    offchain::{
        AppCrypto, SendTransactionTypes, SendUnsignedTransaction, SignedPayload, Signer,
        SigningTypes,
    },
};

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When an offchain worker is signing transactions it's going to request keys from type
/// `KeyTypeId` via the keystore to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).

/// NOTE(maxim): currently we are reusing aura's keys
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"aura");

use codec::{Decode, Encode};

use sp_runtime::RuntimeDebug;

type AString = codec::alloc::string::String;

type WitnessId = Vec<u8>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct WitnessPayload<Public> {
    tx_id: WitnessId,
    public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for WitnessPayload<T::Public> {
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

type CFResult<T> = core::result::Result<T, &'static str>;
pub trait Trait:
    system::Trait + SendTransactionTypes<Call<Self>> + SigningTypes + pallet_cf_validator::Trait
{
    type Call: From<Call<Self>>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OffchainCb {

        WitnessMap get(fn witness_map): map hasher(blake2_128_concat) Vec<u8> => Vec<T::AccountId>;

    }
}

decl_event!(
    /// Events generated by the module.
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        /// New witness is added from a validator
        NewWitness(WitnessId, AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        // Error returned when making unsigned transactions in off-chain worker
        OffchainUnsignedTxError,
        // Error returned when making signed transactions in off-chain worker
        NoLocalAcctForSigning,
        // Error returned when making unsigned transactions with signed payloads in off-chain worker
        OffchainUnsignedTxSignedPayloadError,
        // Error returned when a witness for a validator already exists
        ValidatorAlreadySubmittedWitness,
        // Error returned when the witness is submitted by a non-validator
        InvalidValidator
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 0]
        pub fn add_witness(origin, payload: WitnessPayload<T::Public>,
            _signature: T::Signature) -> DispatchResult
        {
            let _ = ensure_none(origin)?;
            // we don't need to verify the signature here because it has been verified in
            //   `validate_unsigned` function when sending out the unsigned tx.
            let WitnessPayload { tx_id, public } = payload;

            let account = public.into_account();

            if !pallet_cf_validator::Module::<T>::is_validator(&account) {
                let id_str = AString::from_utf8_lossy(&tx_id);
                debug::warn!("👷 OCW rejected witness {} from non-validator {:?}", id_str, account);
                return Err(Error::<T>::InvalidValidator.into());
            }

            Self::add_witness_inner(tx_id.clone(), account)?;

            Ok(())
        }

        fn offchain_worker(block: T::BlockNumber) {

            let s_info = StorageValueRef::persistent(b"witness_fetch::last_seen");

            let last_seen: u64 = match s_info.get() {
                Some(val) => val.expect("Could not decode last_seen value"),
                None => 0
            };

            match fetch_witnesses(last_seen) {

                Ok(witness_res) => {

                    let count = witness_res.data.witness_evts.len();

                    if count > 0 {
                        debug::info!("👷 [{:?}] OCW fetched {} witnesses from CFE", block, count);
                    }

                    for witness in &witness_res.data.witness_evts {

                        let mut tx_id = witness.coin.clone();

                        tx_id.push('-');
                        tx_id.push_str(&witness.transaction_id);
                        let tx_id = tx_id.as_bytes().into();

                        Self::submit_witness(tx_id);
                    }


                    if let Some(witness) = witness_res.data.witness_evts.last() {
                        let last_seen = witness.event_number;

                        if let Err(_) = s_info.mutate::<_, (), _>(|_: Option<Option<u64>>| {
                            Ok(last_seen)
                        }) {
                            debug::error!("👷 OCW was unable to update last_seen");
                        }
                    }

                }
                Err(err) => {
                    debug::error!("👷 OCW was unable to fetch json: {}", err);
                }
            }

        }
    }
}

impl<T: Trait> Module<T> {
    /// returns to the RPC call `get_confirmed_witnesses`
    pub fn get_confirmed_witnesses() -> Vec<Vec<u8>> {
        let mut confirmed_witnesses: Vec<Vec<u8>> = Vec::new();
        let validators = <pallet_cf_validator::Module<T>>::get_validators();
        let num_validators = validators.unwrap_or(Vec::new()).len();
        // super majority
        let threshold = num_validators as f64 * 0.67;
        for (witness_id, validators_of_witness) in <WitnessMap<T>>::iter() {
            if validators_of_witness.len() as f64 > threshold {
                confirmed_witnesses.push(witness_id);
            }
        }
        confirmed_witnesses
    }

    fn add_witness_inner(witness: WitnessId, who: T::AccountId) -> DispatchResult {
        let wstr = AString::from_utf8_lossy(&witness);

        debug::info!("👷 OCW is adding witness {} from {:?}", &wstr, &who);

        if <WitnessMap<T>>::contains_key(&witness) {
            debug::info!("👷 OCW found a record for witness {}", &wstr);

            let mut curr_validators = <WitnessMap<T>>::get(&witness);

            match curr_validators.binary_search(&who) {
                Ok(_) => {
                    debug::info!("👷 OCW already has witness {} from {:?}", &wstr, &who);
                    return Err(Error::<T>::ValidatorAlreadySubmittedWitness.into());
                }
                Err(index) => {
                    debug::info!("👷 OCW inserts witness {} from {:?}", &wstr, &who);
                    curr_validators.insert(index, who.clone());
                    <WitnessMap<T>>::insert(&witness, curr_validators);
                }
            }
        } else {
            debug::info!(
                "👷 OCW can't find a record for witness: {}, creating with validator {:?}",
                &wstr,
                &who
            );

            let validators: Vec<_> = [who.clone()].into();

            <WitnessMap<T>>::insert(&witness, validators);
        }

        Self::deposit_event(RawEvent::NewWitness(witness, who));

        Ok(())
    }

    fn submit_witness(tx_id: WitnessId) {
        // Retrieve the signer to sign the payload
        let signer = Signer::<T, T::AuthorityId>::any_account();

        if let Some((_, res)) = signer.send_unsigned_transaction(
            |acct| WitnessPayload {
                tx_id: tx_id.clone(),
                public: acct.public.clone(),
            },
            Call::add_witness,
        ) {
            if let Err(err) = res {
                debug::error!("OCW failed to submit witness: {:?}", err);
            } else {
                debug::info!("👍 OCW submitted witness successfully!");
            }
        } else {
            debug::error!("No local account available for OCW");
        }
    }
}

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrapper.
/// We can utilize the supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// them with the pallet-specific identifier.
pub mod crypto {
    use crate::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    use sp_runtime::{traits::Verify, MultiSignature, MultiSigner};

    app_crypto!(sr25519, KEY_TYPE);

    pub struct AuthorityId;
    // implemented for ocw-runtime
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthorityId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    // implemented for mock runtime in test
    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for AuthorityId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Witness {
    transaction_id: AString,
    coin: AString,
    event_number: u64,
}

use alt_serde::Deserialize;

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WitnessData {
    witness_evts: Vec<Witness>,
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Debug)]
struct WitnessResponse {
    success: bool,
    data: WitnessData,
}

fn fetch_witnesses(last_seen: u64) -> CFResult<WitnessResponse> {
    // this is used, don't listen to rust-analyzer
    use alt_serde::__private::ToString;

    let mut url = AString::from("http://127.0.0.1:3030/v1/witnesses?last_seen=");
    url.push_str(&last_seen.to_string());
    // for testing, with the json-server
    // let mut url = AString::from("http://127.0.0.1:3000/witnesses");

    debug::info!("[witness]: fetching {}", &url);

    let val = fetch_json(url.as_bytes())?;

    let res = serde_json::from_value(val)
        .map_err(|_| "JSON could not be deserialised to WitnessResponse")?;

    Ok(res)
}

fn fetch_json(remote_url: &[u8]) -> CFResult<serde_json::Value> {
    let remote_url_str =
        core::str::from_utf8(remote_url).map_err(|_| "Error in converting remote_url to string")?;

    let pending = http::Request::get(remote_url_str)
        .send()
        .map_err(|_| "Error in sending http GET request")?;

    let response = pending
        .wait()
        .map_err(|_| "Error in waiting http response back")?;

    if response.code != 200 {
        debug::warn!("Unexpected status code: {}", response.code);
        return Err("Non-200 status code returned from http request");
    }

    let json_buffer: Vec<u8> = response.body().collect::<Vec<u8>>();

    serde_json::from_slice(&json_buffer).map_err(|_| "Response is not a valid json")
}

pub const UNSIGNED_TXS_PRIORITY: u64 = 100;

impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
        let valid_tx = |_provide| {
            ValidTransaction::with_tag_prefix("cf-witness")
                .priority(UNSIGNED_TXS_PRIORITY)
                .longevity(3)
                .propagate(true)
                .and_provides(&[_provide])
                .build()
        };

        match call {
            Call::add_witness(ref payload, ref signature) => {
                // TODO: Check that tx is from a validator
                if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
                    return InvalidTransaction::BadProof.into();
                }
                valid_tx(payload.clone())
            }
            _ => InvalidTransaction::Call.into(),
        }
    }
}
