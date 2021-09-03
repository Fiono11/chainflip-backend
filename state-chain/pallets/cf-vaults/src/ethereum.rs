use crate::ChainParams::Ethereum;
use crate::{
	ChainVault, Config, EthereumVault, Event, Pallet, RequestIndex, RequestResponse,
	ThresholdSignatureRequest, ThresholdSignatureResponse, VaultRotationRequestResponse,
};
use crate::{SchnorrSignature, VaultRotations};
use cf_traits::{NonceIdentifier, NonceProvider, RotationError, VaultRotationHandler};
use ethabi::{Bytes, Function, Param, ParamType, Token};
use frame_support::pallet_prelude::*;
use sp_std::prelude::*;

pub struct EthereumChain<T: Config>(PhantomData<T>);

impl<T: Config> ChainVault for EthereumChain<T> {
	type PublicKey = T::PublicKey;
	type TransactionHash = T::TransactionHash;
	type ValidatorId = T::ValidatorId;
	type Error = RotationError<T::ValidatorId>;

	/// The initial phase has completed with success and we are notified of this from `Vaults`.
	/// Now the specifics for this chain/vault are processed.  In the case for Ethereum we request
	/// to have the function `setAggKeyWithAggKey` signed by the **old** set of validators.
	/// A payload is built and emitted as a `EthSigningTxRequest`, failing this an error is reported
	/// back to `Vaults`
	fn rotate_vault(
		index: RequestIndex,
		new_public_key: Self::PublicKey,
		validators: Vec<Self::ValidatorId>,
	) -> Result<(), Self::Error> {
		// Create payload for signature
		match Self::encode_set_agg_key_with_agg_key(
			new_public_key.clone(),
			SchnorrSignature::default(),
		) {
			Ok(payload) => {
				// Emit the event
				Self::make_request(
					index,
					ThresholdSignatureRequest {
						validators,
						payload,
						public_key: EthereumVault::<T>::get().previous_key,
					},
				)
			}
			Err(_) => {
				Pallet::<T>::abort_rotation();
				Err(RotationError::FailedToConstructPayload)
			}
		}
	}

	/// The vault for this chain has been rotated and we store this vault to storage
	fn vault_rotated(new_public_key: Self::PublicKey, tx_hash: Self::TransactionHash) {
		EthereumVault::<T>::mutate(|vault| {
			(*vault).previous_key = (*vault).current_key.clone();
			(*vault).current_key = new_public_key;
			(*vault).tx_hash = tx_hash;
		});
	}
}

impl<T: Config>
	RequestResponse<
		RequestIndex,
		ThresholdSignatureRequest<T::PublicKey, T::ValidatorId>,
		ThresholdSignatureResponse<T::ValidatorId, SchnorrSignature>,
		RotationError<T::ValidatorId>,
	> for EthereumChain<T>
{
	/// Make the request to sign by emitting an event
	fn make_request(
		index: RequestIndex,
		request: ThresholdSignatureRequest<T::PublicKey, T::ValidatorId>,
	) -> Result<(), RotationError<T::ValidatorId>> {
		Pallet::<T>::deposit_event(Event::ThresholdSignatureRequest(index, request));
		Ok(().into())
	}

	/// Try to handle the response and pass this onto `Vaults` to complete the vault rotation
	fn handle_response(
		index: RequestIndex,
		response: ThresholdSignatureResponse<T::ValidatorId, SchnorrSignature>,
	) -> Result<(), RotationError<T::ValidatorId>> {
		match response {
			ThresholdSignatureResponse::Success(signature) => {
				match VaultRotations::<T>::try_get(index) {
					Ok(vault_rotation) => {
						match Self::encode_set_agg_key_with_agg_key(
							vault_rotation.new_public_key,
							signature,
						) {
							Ok(payload) => {
								// Emit the event
								VaultRotationRequestResponse::<T>::make_request(
									index,
									Ethereum(payload).into(),
								)
							}
							Err(_) => {
								Pallet::<T>::abort_rotation();
								Err(RotationError::FailedToConstructPayload)
							}
						}
					}
					Err(_) => Err(RotationError::InvalidRequestIndex),
				}
			}
			ThresholdSignatureResponse::Error(bad_validators) => {
				T::RotationHandler::penalise(bad_validators.clone());
				Pallet::<T>::abort_rotation();
				Err(RotationError::BadValidators(bad_validators))
			}
		}
	}
}

impl<T: Config> EthereumChain<T> {
	/// Encode `setAggKeyWithAggKey` call using `ethabi`.  This is a long approach as we are working
	/// around `no_std` limitations here for the runtime.
	pub(crate) fn encode_set_agg_key_with_agg_key(
		new_public_key: T::PublicKey,
		signature: SchnorrSignature,
	) -> ethabi::Result<Bytes> {
		Function::new(
			"setAggKeyWithAggKey",
			vec![
				Param::new(
					"sigData",
					ParamType::Tuple(vec![
						ParamType::Uint(256),
						ParamType::Uint(256),
						ParamType::Uint(256),
						ParamType::Address,
					]),
				),
				Param::new("newKey", ParamType::FixedBytes(32)),
			],
			vec![],
			false,
		)
		.encode_input(&vec![
			Token::Tuple(vec![
				Token::Uint(ethabi::Uint::zero()),
				Token::Uint(signature.s.into()),
				Token::Uint(T::NonceProvider::next_nonce(NonceIdentifier::Ethereum).into()),
				Token::Address(signature.r.into()),
			]),
			// newKey: bytes32
			Token::FixedBytes(new_public_key.into()),
		])
	}
}
