mod helpers;

use lazy_static::lazy_static;
use log::*;

use crate::{
    p2p::{P2PMessage, ValidatorId},
    signing::{
        client::{
            client_inner::{
                keygen_state::KeygenStage,
                signing_state::SigningStage,
                tests::helpers::{
                    bc1_to_p2p_signing, generate_valid_keygen_data, keygen_delayed_count,
                    keygen_stage_for, recv_next_signal_message_skipping, sec2_to_p2p_keygen,
                    sec2_to_p2p_signing, sig_to_p2p, signing_delayed_count,
                },
                InnerSignal,
            },
            KeyId, KeygenInfo, MultisigInstruction, SigningInfo, PHASE_TIMEOUT,
        },
        crypto::{Keys, Parameters},
        MessageHash, MessageInfo,
    },
};

use super::client_inner::{
    KeyGenMessage, KeyGenMessageWrapped, MultisigClientInner, MultisigMessage, SigningDataWrapped,
};

// The id to be used by default
const AUCTION_ID: KeyId = KeyId(0);

lazy_static! {
    static ref MESSAGE: Vec<u8> = "Chainflip".as_bytes().to_vec();
    static ref MESSAGE_HASH: MessageHash = MessageHash(MESSAGE.clone());
    static ref MESSAGE_INFO: MessageInfo = MessageInfo {
        hash: MESSAGE_HASH.clone(),
        key_id: AUCTION_ID
    };
    static ref SIGN_INFO: SigningInfo = SigningInfo {
        id: AUCTION_ID,
        signers: vec![1, 2]
    };
    static ref AUCTION_INFO: KeygenInfo = KeygenInfo {
        id: AUCTION_ID,
        signers: vec![1, 2, 3]
    };
}

fn create_bc1(signer_idx: usize) -> Broadcast1 {
    let key = Keys::phase1_create(signer_idx);

    let (bc1, blind) = key.phase1_broadcast();

    let y_i = key.y_i;

    Broadcast1 { bc1, blind, y_i }
}

use std::{sync::Once, time::Duration};

use super::client_inner::Broadcast1;

static INIT: Once = Once::new();

/// Initializes the logger and does only once
/// (doing otherwise would result in error)
fn init_logs_once() {
    INIT.call_once(|| {
        env_logger::builder()
            .format_timestamp(None)
            .format_module_path(false)
            .init();
    })
}

/// After we've received a request to sign, we should immediately be able
/// to receive Broadcast1 messages
#[tokio::test]
async fn should_await_bc1_after_rts() {
    init_logs_once();

    let states = generate_valid_keygen_data().await;

    let mut c1 = states.key_ready.clients[0].clone();

    let key = c1
        .get_keygen()
        .get_key_by_id(AUCTION_ID)
        .expect("no key")
        .to_owned();

    c1.signing_manager
        .on_request_to_sign(MESSAGE_HASH.clone(), key, SIGN_INFO.clone());

    assert_eq!(
        get_stage_for_msg(&c1, &MESSAGE_INFO),
        Some(SigningStage::AwaitingBroadcast1)
    );
}

/// BC1 messages get processed if we receive RTS shortly after
#[tokio::test]
async fn should_process_delayed_bc1_after_rts() {
    init_logs_once();

    let states = generate_valid_keygen_data().await;

    let mut c1 = states.key_ready.clients[0].clone();

    assert!(get_stage_for_msg(&c1, &MESSAGE_INFO).is_none());

    let bc1 = states.sign_phase1.bc1_vec[1].clone();

    let wdata = SigningDataWrapped::new(bc1, MESSAGE_INFO.clone());

    c1.signing_manager.process_signing_data(2, wdata);

    assert_eq!(
        get_stage_for_msg(&c1, &MESSAGE_INFO),
        Some(SigningStage::Idle)
    );

    assert_eq!(signing_delayed_count(&c1, &MESSAGE_INFO), 1);

    let key = c1
        .get_keygen()
        .get_key_by_id(AUCTION_ID)
        .expect("no key")
        .to_owned();

    c1.signing_manager
        .on_request_to_sign(MESSAGE_HASH.clone(), key, SIGN_INFO.clone());

    assert_eq!(signing_delayed_count(&c1, &MESSAGE_INFO), 0);

    assert_eq!(
        get_stage_for_msg(&c1, &MESSAGE_INFO),
        Some(SigningStage::AwaitingSecret2)
    );
}

#[test]
#[ignore = "unimplemented"]
fn signing_data_expire() {
    todo!();
}

fn create_keygen_p2p_message<M>(sender_id: ValidatorId, message: M) -> P2PMessage
where
    M: Into<KeyGenMessage>,
{
    let wrapped = KeyGenMessageWrapped::new(AUCTION_ID, message.into());

    let ms_message = MultisigMessage::from(wrapped);

    let data = serde_json::to_vec(&ms_message).unwrap();

    P2PMessage { sender_id, data }
}

#[test]
fn bc1_gets_delayed_until_keygen_request() {
    let params = Parameters {
        threshold: 1,
        share_count: 3,
    };

    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let mut client = MultisigClientInner::new(1, params, tx, PHASE_TIMEOUT);

    assert_eq!(keygen_stage_for(&client, AUCTION_ID), None);

    let message = create_keygen_p2p_message(2, create_bc1(2));
    client.process_p2p_mq_message(message);

    assert_eq!(keygen_stage_for(&client, AUCTION_ID), None);
    assert_eq!(keygen_delayed_count(&client, AUCTION_ID), 1);

    // Keygen instruction should advance the stage and process delayed messages

    let keygen = MultisigInstruction::KeyGen(AUCTION_INFO.clone());

    client.process_multisig_instruction(keygen);

    assert_eq!(
        keygen_stage_for(&client, AUCTION_ID),
        Some(KeygenStage::AwaitingBroadcast1)
    );
    assert_eq!(keygen_delayed_count(&client, AUCTION_ID), 0);

    // One more message should advance the stage (share_count = 3)
    let message = create_keygen_p2p_message(3, create_bc1(3));
    client.process_p2p_mq_message(message);

    assert_eq!(
        keygen_stage_for(&client, AUCTION_ID),
        Some(KeygenStage::AwaitingSecret2)
    );
}

/// By sending (signing) BC1, a node is trying to start a signing procedure,
/// but we only process it after we've received a signing instruction from
/// our SC. If we don't receive it after a certain period of time, BC1 should
/// be removed and the sender should be penalised.
#[test]
fn delayed_signing_bc1_gets_removed() {
    init_logs_once();
    // Setup
    let params = Parameters {
        threshold: 1,
        share_count: 3,
    };
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let timeout = Duration::from_millis(1);

    let mut client = MultisigClientInner::new(1, params, tx, timeout);

    // Create delayed BC1
    let bc1 = create_bc1(2).into();
    let m = bc1_to_p2p_signing(bc1, 2, &MESSAGE_INFO);
    client.process_p2p_mq_message(m);

    assert_eq!(
        get_stage_for_msg(&client, &MESSAGE_INFO),
        Some(SigningStage::Idle)
    );

    // Wait for the data to expire
    std::thread::sleep(timeout);

    client.cleanup();

    assert_eq!(get_stage_for_msg(&client, &MESSAGE_INFO), None);
}

#[tokio::test]
async fn keygen_secret2_gets_delayed() {
    init_logs_once();

    let states = generate_valid_keygen_data().await;

    // auciton id is always 0 for generate_valid_keygen_data
    let key_id = KeyId(0);

    let phase1 = &states.keygen_phase1;
    let phase2 = &states.keygen_phase2;

    // Note the use of phase2 data on a phase1 client
    let mut clients_p1 = phase1.clients.clone();
    let bc1_vec = phase1.bc1_vec.clone();
    let sec2_vec = phase2.sec2_vec.clone();

    let c1 = &mut clients_p1[0];
    assert_eq!(
        keygen_stage_for(&c1, key_id),
        Some(KeygenStage::AwaitingBroadcast1)
    );

    // Secret sent from client 2 to client 1
    let sec2 = sec2_vec[1].get(&1).unwrap().clone();

    // We should not process it immediately
    let message = create_keygen_p2p_message(2, sec2);

    c1.process_p2p_mq_message(message);

    assert_eq!(keygen_delayed_count(&c1, key_id), 1);
    assert_eq!(
        keygen_stage_for(&c1, key_id),
        Some(KeygenStage::AwaitingBroadcast1)
    );

    // Process incoming bc1_vec, so we can advance to the next phase
    let message = create_keygen_p2p_message(2, bc1_vec[1].clone());
    c1.process_p2p_mq_message(message);

    let message = create_keygen_p2p_message(3, bc1_vec[2].clone());
    c1.process_p2p_mq_message(message);

    assert_eq!(
        keygen_stage_for(&c1, key_id),
        Some(KeygenStage::AwaitingSecret2)
    );
    assert_eq!(keygen_delayed_count(&c1, key_id), 0);
}

#[tokio::test]
async fn signing_secret2_gets_delayed() {
    init_logs_once();

    let states = generate_valid_keygen_data().await;

    let phase1 = &states.sign_phase1;
    let phase2 = &states.sign_phase2;

    // Client in phase1 should be able to receive phase2 data (Secret2)

    let mut c1 = phase1.clients[0].clone();

    assert_eq!(
        get_stage_for_msg(&c1, &MESSAGE_INFO),
        Some(SigningStage::AwaitingBroadcast1)
    );

    let sec2 = phase2.sec2_vec[1].get(&1).unwrap().clone();

    let m = sec2_to_p2p_signing(sec2, 2, &MESSAGE_INFO);

    c1.process_p2p_mq_message(m);

    assert_eq!(
        get_stage_for_msg(&c1, &MESSAGE_INFO),
        Some(SigningStage::AwaitingBroadcast1)
    );

    // Finally c1 receives bc1 and able to advance to phase2
    let bc1 = phase1.bc1_vec[1].clone();

    let m = bc1_to_p2p_signing(bc1, 2, &MESSAGE_INFO);

    c1.process_p2p_mq_message(m);

    // We are able to process delayed secret2 and immediately
    // go from phase1 to phase3
    assert_eq!(
        get_stage_for_msg(&c1, &MESSAGE_INFO),
        Some(SigningStage::AwaitingLocalSig3)
    );
}

#[tokio::test]
async fn signing_local_sig_gets_delayed() {
    init_logs_once();

    let mut states = generate_valid_keygen_data().await;

    let phase2 = &states.sign_phase2;
    let phase3 = &states.sign_phase3;

    let mut c1_p2 = phase2.clients[0].clone();
    let local_sig = phase3.local_sigs[1].clone();

    let m = sig_to_p2p(local_sig, 2, &MESSAGE_INFO);

    c1_p2.process_p2p_mq_message(m);

    assert_eq!(
        get_stage_for_msg(&c1_p2, &MESSAGE_INFO),
        Some(SigningStage::AwaitingSecret2)
    );

    // Send Secret2 to be able to process delayed LocalSig
    let sec2 = phase2.sec2_vec[1].get(&1).unwrap().clone();

    let m = sec2_to_p2p_signing(sec2, 2, &MESSAGE_INFO);

    c1_p2.process_p2p_mq_message(m);

    let s = recv_next_signal_message_skipping(&mut states.rxs[0]).await;

    assert_eq!(Some(InnerSignal::MessageSigned(MESSAGE_INFO.clone())), s);
}

fn get_stage_for_msg(c: &MultisigClientInner, message_info: &MessageInfo) -> Option<SigningStage> {
    c.signing_manager
        .get_state_for(message_info)
        .map(|s| s.get_stage())
}

/// Request to sign should be delayed until the key is ready
#[tokio::test]
async fn request_to_sign_before_key_ready() {
    init_logs_once();

    let key_id = KeyId(0);
    let message_hash = MessageHash(MESSAGE.clone());

    let states = generate_valid_keygen_data().await;

    let mut c1 = states.keygen_phase2.clients[0].clone();

    assert_eq!(
        keygen_stage_for(&c1, key_id),
        Some(KeygenStage::AwaitingSecret2)
    );

    // BC1 for siging arrives before the key is ready
    let bc1_sign = states.sign_phase1.bc1_vec[1].clone();

    let m = bc1_to_p2p_signing(bc1_sign, 2, &MESSAGE_INFO);

    c1.process_p2p_mq_message(m);

    assert_eq!(
        get_stage_for_msg(&c1, &MESSAGE_INFO),
        Some(SigningStage::Idle)
    );

    // Finalize key generation and make sure we can make progress on signing the message

    let sec2_1 = states.keygen_phase2.sec2_vec[1].get(&1).unwrap().clone();
    let m = sec2_to_p2p_keygen(sec2_1, 2);
    c1.process_p2p_mq_message(m);

    let sec2_2 = states.keygen_phase2.sec2_vec[2].get(&1).unwrap().clone();
    let m = sec2_to_p2p_keygen(sec2_2, 3);
    c1.process_p2p_mq_message(m);

    assert_eq!(keygen_stage_for(&c1, key_id), Some(KeygenStage::KeyReady));

    assert_eq!(
        get_stage_for_msg(&c1, &MESSAGE_INFO),
        Some(SigningStage::Idle)
    );

    c1.process_multisig_instruction(MultisigInstruction::Sign(message_hash, SIGN_INFO.clone()));

    // We only need one BC1 (the delayed one) to proceed
    assert_eq!(
        get_stage_for_msg(&c1, &MESSAGE_INFO),
        Some(SigningStage::AwaitingSecret2)
    );
}

#[tokio::test]
#[ignore = "unfinished"]
async fn basic_key_rotation() {
    init_logs_once();

    let states = generate_valid_keygen_data().await;

    error!("TEST BEGINS");

    // Start with clients that already have an aggregate key

    let mut c1 = states.key_ready.clients[0].clone();

    // c1.process_multisig_instruction(MultisigInstruction::KeyGen(Epoch(1)));
}

// INFO: We should be able to continue signing with the old key. When key rotation happens,
// we need to create a new key. A node is likely to remain a validator, so it needs to be
// able to transfer funds from the old key to the new one. SC will send us a command to
// generate a new key for epoch X (and attempt number?). Requests to sign should also
// contain the epoch.

// What needs to be tested (unit tests)
// DONE:
// - Delaying works correctly for Keygen::BC1, Keygen::Secret2, Signing:BC1, Signing::Secret2, Signing::LocalSig
// - BC1 messages are processed after a timely RTS (and can lead to phase 2)
// - RTS is required to proceed to the next phase

// TO DO:
// - Delayed data expires on timeout
// - Signing phases do timeout (only tested for BC1 currently)
// - Parties cannot send two messages for the same phase of signing/keygen
// - When unable to make progress, the state (Signing/Keygen) should be correctly reset
// (i.e. past failures don't impact future signing ceremonies)
// - Should be able to generate new signing keys
// - make sure that we don't process p2p data at index signer_id which is our own

// MAXIM: test that we can't repeat the same key_id
// MAXIM: test that there is no interaction between different key_ids
// MAXIM: test that we clean up states that didn't result in a key
// MAXIM: test that we penalize the offending node
