#[macro_use]
mod utils;
mod ceremony_runner;
mod common;
pub mod key_store_api;
pub mod keygen;
pub mod legacy;
pub mod signing;

#[cfg(test)]
mod helpers;

#[cfg(test)]
mod multisig_client_tests;

pub mod ceremony_manager;

use std::collections::BTreeSet;

use utilities::{format_iterator, threshold_from_share_count};

use cf_primitives::{AuthorityCount, CeremonyId, EpochIndex};
use futures::{future::BoxFuture, FutureExt};
use state_chain_runtime::AccountId;

use serde::{Deserialize, Serialize};

use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, info, info_span, Instrument};

use keygen::KeygenData;

pub use crate::client::utils::PartyIdxMapping;
pub use common::{
	CeremonyFailureReason, KeygenFailureReason, KeygenResult, KeygenResultInfo, KeygenStageName,
	SigningFailureReason,
};

#[cfg(test)]
pub use self::utils::ensure_unsorted;

#[cfg(feature = "test")]
pub use keygen::get_key_data_for_test;

#[cfg(test)]
pub use signing::{gen_signing_data_stage1, gen_signing_data_stage4};

#[cfg(test)]
pub use keygen::{gen_keygen_data_hash_comm1, gen_keygen_data_verify_hash_comm2};

#[cfg(feature = "test")]
use mockall::automock;

use self::{
	ceremony_manager::{CeremonyResultSender, KeygenCeremony, SigningCeremony},
	common::ResharingContext,
	key_store_api::KeyStoreAPI,
	signing::SigningData,
};

use super::{
	crypto::{CanonicalEncoding, CryptoScheme, ECPoint, KeyId},
	Rng,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThresholdParameters {
	/// Total number of key shares (equals the total number of parties in keygen)
	pub share_count: AuthorityCount,
	/// Max number of parties that can *NOT* generate signature
	pub threshold: AuthorityCount,
}

impl ThresholdParameters {
	pub fn from_share_count(share_count: AuthorityCount) -> Self {
		ThresholdParameters { share_count, threshold: threshold_from_share_count(share_count) }
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MultisigData<P: ECPoint> {
	#[serde(bound = "")]
	Keygen(KeygenData<P>),
	#[serde(bound = "")]
	Signing(SigningData<P>),
}

derive_try_from_variant!(impl<P: ECPoint> for KeygenData<P>, MultisigData::Keygen, MultisigData<P>);
derive_try_from_variant!(impl<P: ECPoint> for SigningData<P>, MultisigData::Signing, MultisigData<P>);

impl<P: ECPoint> From<SigningData<P>> for MultisigData<P> {
	fn from(data: SigningData<P>) -> Self {
		MultisigData::Signing(data)
	}
}

impl<P: ECPoint> From<KeygenData<P>> for MultisigData<P> {
	fn from(data: KeygenData<P>) -> Self {
		MultisigData::Keygen(data)
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MultisigMessage<P: ECPoint> {
	ceremony_id: CeremonyId,
	#[serde(bound = "")]
	data: MultisigData<P>,
}

/// The public interface to the multi-signature code
/// The initiate functions of this trait when called send a ceremony request and return a future
/// that can be await'ed on for the result of that ceremony. Splitting requesting and waiting for a
/// ceremony to complete allows the requests to all be sent synchronously which is required as we
/// expect the requests to be ordered by ceremony_id
#[cfg_attr(feature = "test", automock)]
pub trait MultisigClientApi<C: CryptoScheme> {
	fn initiate_keygen(
		&self,
		ceremony_id: CeremonyId,
		epoch_index: EpochIndex,
		participants: BTreeSet<AccountId>,
	) -> BoxFuture<'_, Result<C::PublicKey, (BTreeSet<AccountId>, KeygenFailureReason)>>;

	fn initiate_signing(
		&self,
		ceremony_id: CeremonyId,
		key_id: KeyId,
		signers: BTreeSet<AccountId>,
		payload: Vec<C::SigningPayload>,
	) -> BoxFuture<'_, Result<Vec<C::Signature>, (BTreeSet<AccountId>, SigningFailureReason)>>;

	fn update_latest_ceremony_id(&self, ceremony_id: CeremonyId);
}

/// The ceremony details are optional to alow the updating of the ceremony id tracking
/// when we are not participating in the ceremony.
#[derive(Debug)]
pub struct CeremonyRequest<C: CryptoScheme> {
	pub ceremony_id: CeremonyId,
	pub details: Option<CeremonyRequestDetails<C>>,
}
#[derive(Debug)]
pub enum CeremonyRequestDetails<C>
where
	C: CryptoScheme,
{
	Keygen(KeygenRequestDetails<C>),
	Sign(SigningRequestDetails<C>),
}

#[derive(Debug)]
pub struct KeygenRequestDetails<C: CryptoScheme> {
	pub participants: BTreeSet<AccountId>,
	pub rng: Rng,
	pub result_sender: CeremonyResultSender<KeygenCeremony<C>>,
	/// If not `None`, the participant will use an existing key share
	/// in an attempt to re-share an existing key
	pub resharing_context: Option<ResharingContext<C>>,
}

#[derive(Debug)]
pub struct SigningRequestDetails<C>
where
	C: CryptoScheme,
{
	pub participants: BTreeSet<AccountId>,
	pub payload: Vec<C::SigningPayload>,
	pub keygen_result_info: KeygenResultInfo<C>,
	pub rng: Rng,
	pub result_sender: CeremonyResultSender<SigningCeremony<C>>,
}

/// Multisig client acts as the frontend for the multisig functionality, delegating
/// the actual signing to "Ceremony Manager". It is additionally responsible for
/// persistently storing generated keys and providing them to the signing ceremonies.
pub struct MultisigClient<C: CryptoScheme, KeyStore: KeyStoreAPI<C>> {
	my_account_id: AccountId,
	ceremony_request_sender: UnboundedSender<CeremonyRequest<C>>,
	key_store: std::sync::Mutex<KeyStore>,
}

impl<C: CryptoScheme, KeyStore: KeyStoreAPI<C>> MultisigClient<C, KeyStore> {
	pub fn new(
		my_account_id: AccountId,
		key_store: KeyStore,
		ceremony_request_sender: UnboundedSender<CeremonyRequest<C>>,
	) -> Self {
		MultisigClient {
			my_account_id,
			key_store: std::sync::Mutex::new(key_store),
			ceremony_request_sender,
		}
	}
}

impl<C: CryptoScheme, KeyStore: KeyStoreAPI<C>> MultisigClientApi<C>
	for MultisigClient<C, KeyStore>
{
	fn initiate_keygen(
		&self,
		ceremony_id: CeremonyId,
		// The epoch the key will be associated with if successful.
		epoch_index: EpochIndex,
		participants: BTreeSet<AccountId>,
	) -> BoxFuture<'_, Result<C::PublicKey, (BTreeSet<AccountId>, KeygenFailureReason)>> {
		assert!(participants.contains(&self.my_account_id));
		let span = info_span!("Keygen Ceremony", ceremony_id = ceremony_id);
		let _entered = span.enter();

		info!(
			participants = format_iterator(&participants).to_string(),
			"Received a keygen request"
		);

		use rand_legacy::FromEntropy;
		let rng = Rng::from_entropy();

		let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
		self.ceremony_request_sender
			.send(CeremonyRequest {
				ceremony_id,
				details: Some(CeremonyRequestDetails::Keygen(KeygenRequestDetails {
					participants,
					rng,
					result_sender,
					resharing_context: None,
				})),
			})
			.unwrap();

		async move {
			result_receiver
				.await
				.expect("Keygen result channel dropped before receiving a result")
				.map(|keygen_result_info| {
					let agg_key = keygen_result_info.key.get_agg_public_key();

					self.key_store.lock().unwrap().set_key(
						KeyId { epoch_index, public_key_bytes: agg_key.encode_key() },
						keygen_result_info,
					);
					agg_key
				})
				.map_err(|(reported_parties, failure_reason)| {
					failure_reason.log(&reported_parties);
					(reported_parties, failure_reason)
				})
		}
		.instrument(span.clone())
		.boxed()
	}

	fn initiate_signing(
		&self,
		ceremony_id: CeremonyId,
		key_id: KeyId,
		signers: BTreeSet<AccountId>,
		payload: Vec<C::SigningPayload>,
	) -> BoxFuture<'_, Result<Vec<C::Signature>, (BTreeSet<AccountId>, SigningFailureReason)>> {
		let span = info_span!("Signing Ceremony", ceremony_id = ceremony_id);
		let _entered = span.enter();

		assert!(signers.contains(&self.my_account_id));

		debug!(
			payload_count = payload.len(),
			signers = format_iterator(&signers).to_string(),
			"Received a request to sign",
		);

		use rand_legacy::FromEntropy;
		let rng = Rng::from_entropy();

		if let Some(keygen_result_info) = self.key_store.lock().unwrap().get_key(&key_id) {
			let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
			self.ceremony_request_sender
				.send(CeremonyRequest {
					ceremony_id,
					details: Some(CeremonyRequestDetails::Sign(SigningRequestDetails {
						participants: signers,
						payload,
						keygen_result_info,
						rng,
						result_sender,
					})),
				})
				.unwrap();

			async move {
				// Wait for the request to return a result, then log and return the result
				let result = result_receiver
					.await
					.expect("Signing result oneshot channel dropped before receiving a result");

				result.map_err(|(reported_parties, failure_reason)| {
					failure_reason.log(&reported_parties);

					(reported_parties, failure_reason)
				})
			}
			.boxed()
		} else {
			self.update_latest_ceremony_id(ceremony_id);
			let reported_parties = Default::default();
			let failure_reason = SigningFailureReason::UnknownKey;
			failure_reason.log(&reported_parties);
			futures::future::ready(Err((reported_parties, failure_reason))).boxed()
		}
	}

	fn update_latest_ceremony_id(&self, ceremony_id: CeremonyId) {
		self.ceremony_request_sender
			.send(CeremonyRequest { ceremony_id, details: None })
			.unwrap();
	}
}
