use std::fmt::Display;

use cf_primitives::AuthorityCount;
use serde::{Deserialize, Serialize};

use crate::{
	client::common::{BroadcastVerificationMessage, PreProcessStageDataCheck, SigningStageName},
	crypto::ECPoint,
	ChainTag, CryptoScheme,
};

#[cfg(test)]
pub use tests::{gen_signing_data_stage1, gen_signing_data_stage2, gen_signing_data_stage4};

/// Public components of the single-use nonces generated by
/// a single party at signer index `index`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]

pub struct SigningCommitment<P: ECPoint> {
	#[serde(bound = "")]
	pub d: P,
	#[serde(bound = "")]
	pub e: P,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Comm1<P: ECPoint>(#[serde(bound = "")] pub Vec<SigningCommitment<P>>);

pub type VerifyComm2<P> = BroadcastVerificationMessage<Comm1<P>>;
pub type VerifyLocalSig4<P> = BroadcastVerificationMessage<LocalSig3<P>>;

/// Signature (the "response" part) shard generated by a single party
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalSig3<P: ECPoint> {
	pub responses: Vec<P::Scalar>,
}

/// Data exchanged between parties during various stages
/// of the FROST signing protocol
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SigningData<P: ECPoint> {
	#[serde(bound = "")]
	CommStage1(Comm1<P>),
	#[serde(bound = "")]
	BroadcastVerificationStage2(VerifyComm2<P>),
	#[serde(bound = "")]
	LocalSigStage3(LocalSig3<P>),
	#[serde(bound = "")]
	VerifyLocalSigsStage4(VerifyLocalSig4<P>),
}

derive_impls_for_enum_variants!(impl<P: ECPoint> for Comm1<P>, SigningData::CommStage1, SigningData<P>);
derive_impls_for_enum_variants!(impl<P: ECPoint> for VerifyComm2<P>, SigningData::BroadcastVerificationStage2, SigningData<P>);
derive_impls_for_enum_variants!(impl<P: ECPoint> for LocalSig3<P>, SigningData::LocalSigStage3, SigningData<P>);
derive_impls_for_enum_variants!(impl<P: ECPoint> for VerifyLocalSig4<P>, SigningData::VerifyLocalSigsStage4, SigningData<P>);

derive_display_as_type_name!(Comm1<P: ECPoint>);
derive_display_as_type_name!(VerifyComm2<P: ECPoint>);
derive_display_as_type_name!(LocalSig3<P: ECPoint>);
derive_display_as_type_name!(VerifyLocalSig4<P: ECPoint>);

impl<P: ECPoint> Display for SigningData<P> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let inner = match self {
			SigningData::CommStage1(x) => x.to_string(),
			SigningData::BroadcastVerificationStage2(x) => x.to_string(),
			SigningData::LocalSigStage3(x) => x.to_string(),
			SigningData::VerifyLocalSigsStage4(x) => x.to_string(),
		};
		write!(f, "SigningData({inner})")
	}
}

impl<P: ECPoint> PreProcessStageDataCheck<SigningStageName> for SigningData<P> {
	fn data_size_is_valid<C: CryptoScheme>(&self, num_of_parties: AuthorityCount) -> bool {
		match self {
			SigningData::CommStage1(_) => self.initial_stage_data_size_is_valid::<C>(),
			SigningData::BroadcastVerificationStage2(message) =>
				message.data.len() == num_of_parties as usize,
			SigningData::LocalSigStage3(_) => true,
			SigningData::VerifyLocalSigsStage4(message) =>
				message.data.len() == num_of_parties as usize,
		}
	}

	fn initial_stage_data_size_is_valid<C: CryptoScheme>(&self) -> bool {
		match self {
			SigningData::CommStage1(message) => {
				match C::CHAIN_TAG {
					ChainTag::Ethereum | ChainTag::Polkadot | ChainTag::Ed25519 =>
						message.0.len() == 1,
					// TODO: Find out what a realistic maximum is for the number of payloads we
					// can handle is for btc
					ChainTag::Bitcoin => true,
				}
			},
			_ => panic!("unexpected stage"),
		}
	}

	fn should_delay_unauthorised(&self) -> bool {
		matches!(self, SigningData::CommStage1(_))
	}

	fn should_delay(stage_name: SigningStageName, message: &Self) -> bool {
		match stage_name {
			SigningStageName::AwaitCommitments1 => {
				matches!(message, SigningData::BroadcastVerificationStage2(_))
			},
			SigningStageName::VerifyCommitmentsBroadcast2 => {
				matches!(message, SigningData::LocalSigStage3(_))
			},
			SigningStageName::LocalSigStage3 => {
				matches!(message, SigningData::VerifyLocalSigsStage4(_))
			},
			SigningStageName::VerifyLocalSigsBroadcastStage4 => {
				// Last stage, nothing to delay
				false
			},
		}
	}
}

#[cfg(test)]
mod tests {

	use crate::{
		bitcoin::BtcSigning,
		client::helpers::{gen_dummy_local_sig, gen_dummy_signing_comm1},
		crypto::eth::Point,
		eth::EthSigning,
		polkadot::PolkadotSigning,
		Rng,
	};

	use rand::SeedableRng;

	use super::*;

	pub fn gen_signing_data_stage1(number_of_commitments: u64) -> SigningData<Point> {
		let mut rng = Rng::from_seed([0; 32]);
		SigningData::<Point>::CommStage1(gen_dummy_signing_comm1(&mut rng, number_of_commitments))
	}

	pub fn gen_signing_data_stage2(participant_count: AuthorityCount) -> SigningData<Point> {
		let mut rng = Rng::from_seed([0; 32]);
		SigningData::<Point>::BroadcastVerificationStage2(BroadcastVerificationMessage {
			data: (1..=participant_count)
				.map(|i| (i as AuthorityCount, Some(gen_dummy_signing_comm1(&mut rng, 1))))
				.collect(),
		})
	}

	pub fn gen_signing_data_stage4(participant_count: AuthorityCount) -> SigningData<Point> {
		let mut rng = Rng::from_seed([0; 32]);
		SigningData::<Point>::VerifyLocalSigsStage4(BroadcastVerificationMessage {
			data: (1..=participant_count)
				.map(|i| (i as AuthorityCount, Some(gen_dummy_local_sig(&mut rng))))
				.collect(),
		})
	}

	#[test]
	fn check_data_size_stage1() {
		// Should only pass if the message contains exactly one commitment for ethereum and Polkadot
		assert!(gen_signing_data_stage1(1).initial_stage_data_size_is_valid::<EthSigning>());
		assert!(!gen_signing_data_stage1(0).initial_stage_data_size_is_valid::<EthSigning>());
		assert!(!gen_signing_data_stage1(2).initial_stage_data_size_is_valid::<EthSigning>());
		assert!(!gen_signing_data_stage1(2).initial_stage_data_size_is_valid::<PolkadotSigning>());

		// No limit on bitcoin for now
		assert!(gen_signing_data_stage1(2).initial_stage_data_size_is_valid::<BtcSigning>());
	}

	#[test]
	fn check_data_size_stage2() {
		let test_size = 4;
		let data_to_check = gen_signing_data_stage2(test_size);

		// Should fail on sizes larger or smaller than expected
		assert!(data_to_check.data_size_is_valid::<EthSigning>(test_size));
		assert!(!data_to_check.data_size_is_valid::<EthSigning>(test_size - 1));
		assert!(!data_to_check.data_size_is_valid::<EthSigning>(test_size + 1));
	}

	#[test]
	fn check_data_size_stage4() {
		let test_size = 4;
		let data_to_check = gen_signing_data_stage4(test_size);

		// Should fail on sizes larger or smaller than expected
		assert!(data_to_check.data_size_is_valid::<EthSigning>(test_size));
		assert!(!data_to_check.data_size_is_valid::<EthSigning>(test_size - 1));
		assert!(!data_to_check.data_size_is_valid::<EthSigning>(test_size + 1));
	}

	#[test]
	fn should_delay_correct_data_for_stage() {
		let mut rng = Rng::from_seed([0; 32]);
		let default_length = 1;

		let stage_name = [
			SigningStageName::AwaitCommitments1,
			SigningStageName::VerifyCommitmentsBroadcast2,
			SigningStageName::LocalSigStage3,
			SigningStageName::VerifyLocalSigsBroadcastStage4,
		];
		let stage_data = [
			gen_signing_data_stage1(default_length as u64),
			gen_signing_data_stage2(default_length),
			SigningData::<Point>::LocalSigStage3(gen_dummy_local_sig(&mut rng)),
			gen_signing_data_stage4(default_length),
		];

		for (stage_index, name) in stage_name.iter().enumerate() {
			for (data_index, data) in stage_data.iter().enumerate() {
				if stage_index + 1 == data_index {
					// Should delay the next stage data (stage_index + 1)
					assert!(SigningData::should_delay(*name, data));
				} else {
					// Should not delay any other stage
					assert!(!SigningData::should_delay(*name, data));
				}
			}
		}
	}
}