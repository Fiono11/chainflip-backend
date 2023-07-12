use std::sync::Arc;

use cf_chains::{
	eth::{SchnorrVerificationComponents, TransactionFee},
	Ethereum,
};
use ethers::{
	abi::ethereum_types::BloomInput,
	types::{Bloom, Log, TransactionReceipt},
};
use sp_core::{H160, H256, U256};
use state_chain_runtime::EthereumInstance;
use tracing::{info, trace};

use crate::{
	eth::{key_manager::*, retry_rpc::EthersRetryRpcApi},
	state_chain_observer::client::extrinsic_api::signed::SignedExtrinsicApi,
};

use super::{
	chain_source::{ChainClient, Header},
	chunked_chain_source::{
		chunked_by_vault::{ChunkedByVault, ChunkedByVaultAlias, Generic},
		Builder,
	},
};

use anyhow::{anyhow, Result};

use ethers::abi::RawLog;

use std::fmt::Debug;

/// Type for storing common (i.e. tx_hash) and specific event information
#[derive(Debug, PartialEq, Eq)]
pub struct Event<EventParameters: Debug> {
	/// The transaction hash of the transaction that emitted this event
	pub tx_hash: H256,
	/// The index number of this particular log, in the list of logs emitted by the tx_hash
	pub log_index: U256,
	/// The event specific parameters
	pub event_parameters: EventParameters,
}

impl<EventParameters: Debug> std::fmt::Display for Event<EventParameters> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "EventParameters: {:?}; tx_hash: {:#x}", self.event_parameters, self.tx_hash)
	}
}

impl<EventParameters: Debug + ethers::contract::EthLogDecode> Event<EventParameters> {
	pub fn new_from_unparsed_logs(log: Log) -> Result<Self> {
		Ok(Self {
			tx_hash: log
				.transaction_hash
				.ok_or_else(|| anyhow!("Could not get transaction hash from ETH log"))?,
			log_index: log
				.log_index
				.ok_or_else(|| anyhow!("Could not get log index from ETH log"))?,
			event_parameters: EventParameters::decode_log(&RawLog {
				topics: log.topics.into_iter().collect(),
				data: log.data.to_vec(),
			})?,
		})
	}
}

pub async fn events_at_block<EventParameters, EthRpcClient>(
	header: Header<u64, H256, Bloom>,
	contract_address: H160,
	eth_rpc: &EthRpcClient,
) -> Result<Vec<Event<EventParameters>>>
where
	EventParameters: std::fmt::Debug + ethers::contract::EthLogDecode + Send + Sync + 'static,
	EthRpcClient: EthersRetryRpcApi + ChainClient,
{
	let mut contract_bloom = Bloom::default();
	contract_bloom.accrue(BloomInput::Raw(&contract_address.0));

	// if we have logs for this block, fetch them.
	if header.data.contains_bloom(&contract_bloom) {
		eth_rpc
			.get_logs(header.hash, contract_address)
			.await
			.into_iter()
			.map(|unparsed_log| -> anyhow::Result<Event<EventParameters>> {
				Event::<EventParameters>::new_from_unparsed_logs(unparsed_log)
			})
			.collect::<anyhow::Result<Vec<_>>>()
	} else {
		// we know there won't be interesting logs, so don't fetch for events
		anyhow::Result::Ok(vec![])
	}
}

impl<Inner: ChunkedByVault> Builder<Generic<Inner>> {
	pub fn key_manager_witnessing<
		StateChainClient,
		EthRpcClient: EthersRetryRpcApi + ChainClient + Clone,
	>(
		self,
		state_chain_client: Arc<StateChainClient>,
		eth_rpc: EthRpcClient,
		contract_address: H160,
	) -> Builder<impl ChunkedByVaultAlias>
	where
		Inner: ChunkedByVault<Index = u64, Hash = H256, Data = Bloom, Chain = Ethereum>,
		StateChainClient: SignedExtrinsicApi + Send + Sync + 'static,
	{
		self.map::<Result<Bloom>, _, _>(move |epoch, header| {
			let state_chain_client = state_chain_client.clone();
			let eth_rpc = eth_rpc.clone();
			async move {
				for event in
					events_at_block::<KeyManagerEvents, _>(header, contract_address, &eth_rpc)
						.await?
				{
					info!("Handling event: {event}");
					match event.event_parameters {
						KeyManagerEvents::AggKeySetByAggKeyFilter(_) => {
							state_chain_client
									.submit_signed_extrinsic(pallet_cf_witnesser::Call::witness_at_epoch {
										call: Box::new(
											pallet_cf_vaults::Call::<_, EthereumInstance>::vault_key_rotated {
												block_number: header.index,
												tx_id: event.tx_hash,
											}
											.into(),
										),
										epoch_index: epoch.index,
									})
									.await;
						},
						KeyManagerEvents::AggKeySetByGovKeyFilter(AggKeySetByGovKeyFilter {
							new_agg_key,
							..
						}) => {
							state_chain_client
									.submit_signed_extrinsic(
										pallet_cf_witnesser::Call::witness_at_epoch {
											call: Box::new(
												pallet_cf_vaults::Call::<_, EthereumInstance>::vault_key_rotated_externally {
													new_public_key:
														cf_chains::eth::AggKey::from_pubkey_compressed(
															new_agg_key.serialize(),
														),
													block_number: header.index,
													tx_id: event.tx_hash,
												}
												.into(),
											),
											epoch_index: epoch.index,
										},
									)
									.await;
						},
						KeyManagerEvents::SignatureAcceptedFilter(SignatureAcceptedFilter {
							sig_data,
							..
						}) => {
							let TransactionReceipt { gas_used, effective_gas_price, from, .. } =
								eth_rpc.transaction_receipt(event.tx_hash).await;

							let gas_used = gas_used
								.ok_or_else(|| {
									anyhow::anyhow!(
										"No gas_used on Transaction receipt for tx_hash: {}",
										event.tx_hash
									)
								})?
								.try_into()
								.map_err(anyhow::Error::msg)?;
							let effective_gas_price = effective_gas_price
								.ok_or_else(|| {
									anyhow::anyhow!(
										"No effective_gas_price on Transaction receipt for tx_hash: {}"
									, event.tx_hash)
								})?
								.try_into()
								.map_err(anyhow::Error::msg)?;
							state_chain_client
									.submit_signed_extrinsic(
										pallet_cf_witnesser::Call::witness_at_epoch {
											call: Box::new(
												pallet_cf_broadcast::Call::<_, EthereumInstance>::transaction_succeeded {
													tx_out_id: SchnorrVerificationComponents {
														s: sig_data.sig.into(),
														k_times_g_address: sig_data.k_times_g_address.into(),
													},
													signer_id: from,
													tx_fee: TransactionFee { effective_gas_price, gas_used },
												}
												.into(),
											),
											epoch_index: epoch.index,
										},
									)
									.await;
						},
						KeyManagerEvents::GovernanceActionFilter(GovernanceActionFilter {
							message,
						}) => {
							state_chain_client
								.submit_signed_extrinsic(
									pallet_cf_witnesser::Call::witness_at_epoch {
										call: Box::new(
											pallet_cf_governance::Call::set_whitelisted_call_hash {
												call_hash: message,
											}
											.into(),
										),
										epoch_index: epoch.index,
									},
								)
								.await;
						},
						_ => {
							trace!("Ignoring unused event: {event}");
						},
					}
				}

				Result::Ok(header.data)
			}
		})
	}
}

#[cfg(test)]
mod tests {
	use std::{path::PathBuf, str::FromStr};

	use cf_primitives::AccountRole;
	use futures_util::FutureExt;
	use sp_core::H160;
	use utilities::task_scope::task_scope;

	use crate::{
		eth::{
			ethers_rpc::{EthersRpcApi, EthersRpcClient},
			retry_rpc::EthersRetryRpcClient,
		},
		settings::{self},
		state_chain_observer::client::StateChainClient,
		witness::{
			chain_source::{eth_source::EthSource, extension::ChainSourceExt},
			epoch_source::EpochSource,
		},
	};

	#[ignore = "requires connection to live network"]
	#[tokio::test]
	async fn test_key_manager_witnesser() {
		task_scope(|scope| {
			async {
				let eth_settings = settings::Eth {
					ws_node_endpoint: "ws://localhost:8546".to_string(),
					http_node_endpoint: "http://localhost:8545".to_string(),
					private_key_file: PathBuf::from_str(
						"/Users/kylezs/Documents/test-keys/eth-cf-metamask",
					)
					.unwrap(),
				};

				let client = EthersRpcClient::new(&eth_settings).await.unwrap();

				let chain_id = client.chain_id().await.unwrap();
				println!("Here's the chain_id: {chain_id}");

				let retry_client = EthersRetryRpcClient::new(
					scope,
					client,
					eth_settings.ws_node_endpoint,
					web3::types::U256::from(10997),
				);

				let (state_chain_stream, state_chain_client) =
					StateChainClient::connect_with_account(
						scope,
						"ws://localhost:9944",
						PathBuf::from_str("/Users/kylezs/Documents/test-keys/bashful-key")
							.unwrap()
							.as_path(),
						AccountRole::None,
						false,
					)
					.await
					.unwrap();

				let vault_source =
					EpochSource::new(scope, state_chain_stream, state_chain_client.clone())
						.await
						.vaults()
						.await;

				EthSource::new(retry_client.clone())
					.chunk_by_vault(vault_source)
					.await
					.key_manager_witnessing(
						state_chain_client,
						retry_client,
						H160::from_str("a16e02e87b7454126e5e10d957a927a7f5b5d2be").unwrap(),
					)
					.run()
					.await;

				Ok(())
			}
			.boxed()
		})
		.await
		.unwrap();
	}
}