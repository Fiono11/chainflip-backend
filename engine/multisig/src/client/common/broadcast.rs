use std::{
	collections::{btree_map, BTreeMap},
	fmt::Display,
};

use async_trait::async_trait;
use cf_primitives::{AuthorityCount, CeremonyId};
use tracing::warn;

use crate::{
	client::{ceremony_manager::CeremonyTrait, MultisigMessage},
	p2p::{OutgoingMultisigStageMessages, ProtocolVersion, CURRENT_PROTOCOL_VERSION},
};

use super::ceremony_stage::{CeremonyCommon, CeremonyStage, ProcessMessageResult, StageResult};

pub use super::broadcast_verification::verify_broadcasts;

/// Used by individual stages to distinguish between
/// a public message that should be broadcast to everyone
/// an secret messages that should be delivered to different
/// parties in private
pub enum DataToSend<T> {
	Broadcast(T),
	Private(BTreeMap<AuthorityCount, T>),
}

/// Abstracts away computations performed during every "broadcast" stage
/// of a ceremony
#[async_trait]
pub trait BroadcastStageProcessor<C: CeremonyTrait>: Display {
	/// The specific variant of D shared between parties
	/// during this stage
	type Message: Clone + Into<C::Data> + TryFrom<C::Data, Error = C::Data> + Send;

	/// Unique stage name used for logging and testing.
	const NAME: C::CeremonyStageName;

	/// Init the stage, returning the data to broadcast
	fn init(&mut self) -> DataToSend<Self::Message>;

	/// Determines how the data for this stage (of type `Self::Message`)
	/// should be processed once it either received it from all other parties
	/// or the stage timed out (None is used for missing messages)
	async fn process(
		self,
		messages: BTreeMap<AuthorityCount, Option<Self::Message>>,
	) -> StageResult<C>;
}

/// Responsible for broadcasting/collecting of stage data,
/// delegating the actual processing to `StageProcessor`
pub struct BroadcastStage<C: CeremonyTrait, Stage>
where
	Stage: BroadcastStageProcessor<C>,
{
	common: CeremonyCommon,
	/// Messages collected so far
	messages: BTreeMap<AuthorityCount, Stage::Message>,
	/// Determines the actual computations before/after
	/// the data is collected
	processor: Stage,
}

impl<C: CeremonyTrait, Stage> BroadcastStage<C, Stage>
where
	Stage: BroadcastStageProcessor<C>,
{
	pub fn new(processor: Stage, common: CeremonyCommon) -> Self {
		BroadcastStage { common, messages: BTreeMap::new(), processor }
	}
}

impl<C: CeremonyTrait, Stage> Display for BroadcastStage<C, Stage>
where
	Stage: BroadcastStageProcessor<C>,
	BroadcastStage<C, Stage>: CeremonyStage<C>,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "BroadcastStage({})", &self.get_stage_name())
	}
}

fn serialize_for_version<C: CeremonyTrait>(
	ceremony_id: CeremonyId,
	data: C::Data,
	version: ProtocolVersion,
) -> Vec<u8> {
	let message = MultisigMessage { ceremony_id, data: data.into() };
	match version {
		1 => bincode::serialize(&message).unwrap(),
		_ => panic!("Unsupported protocol version"),
	}
}

#[async_trait]
impl<C: CeremonyTrait, Stage> CeremonyStage<C> for BroadcastStage<C, Stage>
where
	Stage: BroadcastStageProcessor<C> + Send,
{
	fn init(&mut self) {
		let common = &self.common;

		let idx_to_id = |idx: &AuthorityCount| common.validator_mapping.get_id(*idx).clone();

		let (own_message, outgoing_messages) = match self.processor.init() {
			DataToSend::Broadcast(stage_data) => {
				let ceremony_data: C::Data = stage_data.clone().into();
				(
					stage_data,
					OutgoingMultisigStageMessages::Broadcast(
						common
							.all_idxs
							.iter()
							.filter(|idx| **idx != common.own_idx)
							.map(idx_to_id)
							.collect(),
						serialize_for_version::<C>(
							common.ceremony_id,
							ceremony_data,
							CURRENT_PROTOCOL_VERSION,
						),
					),
				)
			},
			DataToSend::Private(mut messages) => (
				messages.remove(&common.own_idx).expect("Must include message to self"),
				OutgoingMultisigStageMessages::Private(
					messages
						.into_iter()
						.map(|(idx, stage_data)| {
							let ceremony_data: C::Data = stage_data.into();
							(
								idx_to_id(&idx),
								serialize_for_version::<C>(
									common.ceremony_id,
									ceremony_data,
									CURRENT_PROTOCOL_VERSION,
								),
							)
						})
						.collect(),
				),
			),
		};

		// Save our own share
		self.messages.insert(common.own_idx, own_message);

		self.common
			.outgoing_p2p_message_sender
			.send(outgoing_messages)
			.expect("Could not send p2p message.");
	}

	fn process_message(&mut self, signer_idx: AuthorityCount, m: C::Data) -> ProcessMessageResult {
		let m: Stage::Message = match m.try_into() {
			Ok(m) => m,
			Err(incorrect_type) => {
				warn!(
					from_id = self.common.validator_mapping.get_id(signer_idx).to_string(),
					"Ignoring unexpected message {incorrect_type} while in stage {self}",
				);
				return ProcessMessageResult::NotReady
			},
		};

		if !self.common.all_idxs.contains(&signer_idx) {
			warn!(
				from_id = self.common.validator_mapping.get_id(signer_idx).to_string(),
				"Ignoring a message from non-participant for stage {self}",
			);
			return ProcessMessageResult::NotReady
		}

		match self.messages.entry(signer_idx) {
			btree_map::Entry::Occupied(_) => {
				warn!(
					from_id = self.common.validator_mapping.get_id(signer_idx).to_string(),
					"Ignoring a redundant message for stage {self}",
				);
				ProcessMessageResult::NotReady
			},
			btree_map::Entry::Vacant(entry) => {
				entry.insert(m);

				if self.messages.len() == self.common.all_idxs.len() {
					ProcessMessageResult::Ready
				} else {
					ProcessMessageResult::NotReady
				}
			},
		}
	}

	async fn finalize(mut self: Box<Self>) -> StageResult<C> {
		// Because we might want to finalize the stage before
		// all data has been received (e.g. due to a timeout),
		// we insert None for any missing data

		let mut received_messages = std::mem::take(&mut self.messages);

		// Turns values T into Option<T>, inserting `None` where
		// data hasn't been received for `idx`
		let messages: BTreeMap<_, _> = self
			.common
			.all_idxs
			.iter()
			.map(|idx| (*idx, received_messages.remove(idx)))
			.collect();

		self.processor.process(messages).await
	}

	fn awaited_parties(&self) -> std::collections::BTreeSet<AuthorityCount> {
		let mut awaited = std::collections::BTreeSet::new();

		for idx in &self.common.all_idxs {
			if !self.messages.contains_key(idx) {
				awaited.insert(*idx);
			}
		}

		awaited
	}

	fn get_stage_name(&self) -> C::CeremonyStageName {
		<Stage as BroadcastStageProcessor<C>>::NAME
	}

	fn ceremony_common(&self) -> &CeremonyCommon {
		&self.common
	}
}