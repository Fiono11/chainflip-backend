#[macro_use]
mod utils;
mod common;
mod key_store;
pub mod keygen;
mod keygen_state_runner;
pub mod signing;
mod state_runner;

#[cfg(test)]
mod tests;

mod ceremony_manager;

#[cfg(test)]
mod genesis;

use std::{collections::HashMap, time::Instant};

use crate::{
    eth::utils::pubkey_to_eth_addr,
    logging::{CEREMONY_ID_KEY, REQUEST_TO_SIGN_EXPIRED},
    multisig::{KeyDB, KeyId, MultisigInstruction},
};

use state_chain_runtime::AccountId;

use serde::{Deserialize, Serialize};

use pallet_cf_vaults::CeremonyId;

use key_store::KeyStore;

use tokio::sync::mpsc::UnboundedSender;
use utilities::threshold_from_share_count;

use keygen::KeygenData;

pub use common::KeygenResultInfo;

#[cfg(test)]
pub use utils::ensure_unsorted;

use self::{
    ceremony_manager::CeremonyManager,
    signing::{frost::SigningData, PendingSigningInfo},
};

pub use keygen::KeygenOptions;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchnorrSignature {
    /// Scalar component
    pub s: [u8; 32],
    /// Point component (commitment)
    pub r: secp256k1::PublicKey,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThresholdParameters {
    /// Total number of key shares (equals the total number of parties in keygen)
    pub share_count: usize,
    /// Max number of parties that can *NOT* generate signature
    pub threshold: usize,
}

impl ThresholdParameters {
    pub fn from_share_count(share_count: usize) -> Self {
        ThresholdParameters {
            share_count,
            threshold: threshold_from_share_count(share_count as u32) as usize,
        }
    }
}

impl From<SchnorrSignature> for cf_chains::eth::SchnorrVerificationComponents {
    fn from(cfe_sig: SchnorrSignature) -> Self {
        Self {
            s: cfe_sig.s,
            k_times_g_addr: pubkey_to_eth_addr(cfe_sig.r),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MultisigData {
    Keygen(KeygenData),
    Signing(SigningData),
}

impl From<SigningData> for MultisigData {
    fn from(data: SigningData) -> Self {
        MultisigData::Signing(data)
    }
}

impl From<KeygenData> for MultisigData {
    fn from(data: KeygenData) -> Self {
        MultisigData::Keygen(data)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MultisigMessage {
    ceremony_id: CeremonyId,
    data: MultisigData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CeremonyAbortReason {
    Unauthorised,
    Timeout,
    Invalid,
}

pub type CeremonyOutcomeResult<Output> = Result<Output, (CeremonyAbortReason, Vec<AccountId>)>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CeremonyOutcome<Id, Output> {
    pub id: Id,
    pub result: CeremonyOutcomeResult<Output>,
}
impl<Id, Output> CeremonyOutcome<Id, Output> {
    pub fn success(id: Id, output: Output) -> Self {
        Self {
            id,
            result: Ok(output),
        }
    }
    pub fn unauthorised(id: Id, bad_validators: Vec<AccountId>) -> Self {
        Self {
            id,
            result: Err((CeremonyAbortReason::Unauthorised, bad_validators)),
        }
    }
    pub fn timeout(id: Id, bad_validators: Vec<AccountId>) -> Self {
        Self {
            id,
            result: Err((CeremonyAbortReason::Timeout, bad_validators)),
        }
    }
    pub fn invalid(id: Id, bad_validators: Vec<AccountId>) -> Self {
        Self {
            id,
            result: Err((CeremonyAbortReason::Invalid, bad_validators)),
        }
    }
}

/// The final result of a keygen ceremony
pub type KeygenOutcome = CeremonyOutcome<CeremonyId, secp256k1::PublicKey>;
/// The final result of a Signing ceremony
pub type SigningOutcome = CeremonyOutcome<CeremonyId, SchnorrSignature>;

pub type MultisigOutcomeSender = tokio::sync::mpsc::UnboundedSender<MultisigOutcome>;

#[derive(Debug, Serialize, Deserialize)]
pub enum MultisigOutcome {
    Signing(SigningOutcome),
    Keygen(KeygenOutcome),
    Ignore,
}

/// Multisig client is is responsible for persistently storing generated keys and
/// delaying signing requests (delegating the actual ceremony management to sub components)
#[derive(Clone)]
pub struct MultisigClient<S>
where
    S: KeyDB,
{
    my_account_id: AccountId,
    key_store: KeyStore<S>,
    pub ceremony_manager: CeremonyManager,
    multisig_outcome_sender: MultisigOutcomeSender,
    outgoing_p2p_message_sender: UnboundedSender<(AccountId, MultisigMessage)>,
    /// Requests awaiting a key
    pending_requests_to_sign: HashMap<KeyId, Vec<PendingSigningInfo>>,
    keygen_options: KeygenOptions,
    logger: slog::Logger,
}

impl<S> MultisigClient<S>
where
    S: KeyDB,
{
    pub fn new(
        my_account_id: AccountId,
        db: S,
        multisig_outcome_sender: MultisigOutcomeSender,
        outgoing_p2p_message_sender: UnboundedSender<(AccountId, MultisigMessage)>,
        keygen_options: KeygenOptions,
        logger: &slog::Logger,
    ) -> Self {
        MultisigClient {
            my_account_id: my_account_id.clone(),
            key_store: KeyStore::new(db),
            ceremony_manager: CeremonyManager::new(
                my_account_id,
                multisig_outcome_sender.clone(),
                outgoing_p2p_message_sender.clone(),
                logger,
            ),
            multisig_outcome_sender,
            outgoing_p2p_message_sender,
            pending_requests_to_sign: Default::default(),
            keygen_options,
            logger: logger.clone(),
        }
    }

    /// Clean up expired states
    pub fn cleanup(&mut self) {
        slog::trace!(self.logger, "Cleaning up multisig states");
        self.ceremony_manager.cleanup();

        // cleanup stale signing_info in pending_requests_to_sign
        let logger = &self.logger;
        self.pending_requests_to_sign
            .retain(|key_id, pending_signing_infos| {
                pending_signing_infos.retain(|pending| {
                    if pending.should_expire_at < Instant::now() {
                        slog::warn!(
                            logger,
                            #REQUEST_TO_SIGN_EXPIRED,
                            "Request to sign expired waiting for key id: {:?}",
                            key_id;
                            CEREMONY_ID_KEY => pending.signing_info.ceremony_id,
                        );
                        return false;
                    }
                    true
                });
                !pending_signing_infos.is_empty()
            });
    }

    /// Process `instruction` issued internally (i.e. from SC or another local module)
    pub fn process_multisig_instruction(&mut self, instruction: MultisigInstruction) {
        match instruction {
            MultisigInstruction::Keygen(keygen_info) => {
                // For now disable generating a new key when we already have one

                slog::debug!(
                    self.logger,
                    "Received a keygen request, participants: {:?}",
                    keygen_info.signers;
                    CEREMONY_ID_KEY => keygen_info.ceremony_id
                );

                self.ceremony_manager
                    .on_keygen_request(keygen_info, self.keygen_options);
            }
            MultisigInstruction::Sign(sign_info) => {
                let key_id = &sign_info.key_id;

                slog::debug!(
                    self.logger,
                    "Received a request to sign, message_hash: {}, signers: {:?}",
                    sign_info.data, sign_info.signers;
                    CEREMONY_ID_KEY => sign_info.ceremony_id
                );
                match self.key_store.get_key(key_id) {
                    Some(keygen_result_info) => {
                        self.ceremony_manager.on_request_to_sign(
                            sign_info.data,
                            keygen_result_info.clone(),
                            sign_info.signers,
                            sign_info.ceremony_id,
                        );
                    }
                    None => {
                        // The key is not ready, delay until either it is ready or timeout

                        slog::debug!(
                            self.logger,
                            "Delaying a request to sign for unknown key: {:?}",
                            sign_info.key_id;
                            CEREMONY_ID_KEY => sign_info.ceremony_id
                        );

                        self.pending_requests_to_sign
                            .entry(sign_info.key_id.clone())
                            .or_default()
                            .push(PendingSigningInfo::new(sign_info));
                    }
                }
            }
        }
    }

    /// Process requests to sign that required the key in `key_info`
    fn process_pending_requests_to_sign(&mut self, key_info: KeygenResultInfo) {
        if let Some(reqs) = self
            .pending_requests_to_sign
            .remove(&KeyId(key_info.key.get_public_key_bytes()))
        {
            for pending in reqs {
                let signing_info = pending.signing_info;
                slog::debug!(
                    self.logger,
                    "Processing a pending requests to sign";
                    CEREMONY_ID_KEY => signing_info.ceremony_id
                );

                self.ceremony_manager.on_request_to_sign(
                    signing_info.data,
                    key_info.clone(),
                    signing_info.signers,
                    signing_info.ceremony_id,
                )
            }
        }
    }

    fn on_key_generated(&mut self, ceremony_id: CeremonyId, key_info: KeygenResultInfo) {
        self.key_store
            .set_key(KeyId(key_info.key.get_public_key_bytes()), key_info.clone());
        self.process_pending_requests_to_sign(key_info.clone());

        // NOTE: we only notify the SC after we have successfully saved the key

        if let Err(err) =
            self.multisig_outcome_sender
                .send(MultisigOutcome::Keygen(KeygenOutcome::success(
                    ceremony_id,
                    key_info.key.get_public_key().get_element(),
                )))
        {
            // TODO: alert
            slog::error!(
                self.logger,
                "Could not sent KeygenOutcome::Success: {}",
                err
            );
        }
    }

    /// Process message from another validator
    pub fn process_p2p_message(&mut self, sender_id: AccountId, message: MultisigMessage) {
        match message {
            MultisigMessage {
                ceremony_id,
                data: MultisigData::Keygen(data),
            } => {
                // NOTE: we should be able to process Keygen messages
                // even when we are "signing"... (for example, if we want to
                // generate a new key)

                if let Some(key) =
                    self.ceremony_manager
                        .process_keygen_data(sender_id, ceremony_id, data)
                {
                    self.on_key_generated(ceremony_id, key);
                    // NOTE: we could already delete the state here, but it is
                    // not necessary as it will be deleted by "cleanup"
                }
            }
            MultisigMessage {
                ceremony_id,
                data: MultisigData::Signing(data),
            } => {
                // NOTE: we should be able to process Signing messages
                // even when we are generating a new key (for example,
                // we should be able to receive phase1 messages before we've
                // finalized the signing key locally)
                self.ceremony_manager
                    .process_signing_data(sender_id, ceremony_id, data);
            }
        }
    }
}

#[cfg(test)]
impl<S> MultisigClient<S>
where
    S: KeyDB,
{
    pub fn get_key(&self, key_id: &KeyId) -> Option<&KeygenResultInfo> {
        self.key_store.get_key(key_id)
    }

    pub fn get_db(&self) -> &S {
        self.key_store.get_db()
    }

    pub fn get_my_account_id(&self) -> AccountId {
        self.my_account_id.clone()
    }

    /// Change the time we wait until deleting all unresolved states
    pub fn expire_all(&mut self) {
        self.ceremony_manager.expire_all();

        self.pending_requests_to_sign.retain(|_, pending_infos| {
            for pending in pending_infos {
                pending.set_expiry_time(std::time::Instant::now());
            }
            true
        });
    }
}
