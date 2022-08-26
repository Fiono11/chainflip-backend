use std::{sync::Arc, time::Duration};

use crate::state_chain_observer::client::{StateChainClient, StateChainRpcApi};

use super::{rpc::EthRpcApi, EpochStart};

use cf_chains::{eth::TrackedData, Ethereum};

use sp_core::U256;
use state_chain_runtime::CfeSettings;
use tokio::sync::{broadcast, watch};
use utilities::{context, make_periodic_tick};
use web3::types::{BlockNumber, U64};

const ETH_CHAIN_TRACKING_POLL_INTERVAL: Duration = Duration::from_secs(4);

pub async fn start<EthRpcClient, ScRpcClient>(
    eth_rpc: EthRpcClient,
    state_chain_client: Arc<StateChainClient<ScRpcClient>>,
    epoch_start_receiver: broadcast::Receiver<EpochStart>,
    cfe_settings_update_receiver: watch::Receiver<CfeSettings>,
    logger: &slog::Logger,
) -> anyhow::Result<()>
where
    EthRpcClient: 'static + EthRpcApi + Clone + Send + Sync,
    ScRpcClient: 'static + StateChainRpcApi + Send + Sync,
{
    super::epoch_witnesser::start(
        "ETH-Chain-Data",
        epoch_start_receiver,
        |epoch_start| epoch_start.current,
        None,
        move |
            end_witnessing_signal,
            _epoch_start,
            mut last_witnessed_block_hash,
            logger
        | {
            let eth_rpc = eth_rpc.clone();
            let cfe_settings_update_receiver = cfe_settings_update_receiver.clone();

            let state_chain_client = state_chain_client.clone();
            async move {
                let mut poll_interval = make_periodic_tick(ETH_CHAIN_TRACKING_POLL_INTERVAL, false);

                loop {
                    if let Some(_end_block) = *end_witnessing_signal.lock().unwrap() {
                        break;
                    }

                    let block_number = eth_rpc.block_number().await?;
                    let block_hash = context!(eth_rpc.block(block_number).await?.hash)?;
                    if last_witnessed_block_hash != Some(block_hash) {
                        let priority_fee = cfe_settings_update_receiver
                            .borrow()
                            .eth_priority_fee_percentile;
                        let _result = state_chain_client
                            .submit_signed_extrinsic(
                                state_chain_runtime::Call::Witnesser(pallet_cf_witnesser::Call::witness {
                                    call: Box::new(state_chain_runtime::Call::EthereumChainTracking(
                                        pallet_cf_chain_tracking::Call::update_chain_state {
                                            state: get_tracked_data(
                                                &eth_rpc,
                                                block_number.as_u64(),
                                                priority_fee
                                            ).await?,
                                        },
                                    )),
                                }),
                                &logger,
                            )
                            .await;

                        last_witnessed_block_hash = Some(block_hash);
                    }

                    poll_interval.tick().await;
                }

                Ok(last_witnessed_block_hash)
            }
        },
        logger,
    ).await
}

/// Queries the rpc node and builds the `TrackedData` for Ethereum at the requested block number.
///
/// Value in Wei is rounded to nearest Gwei in an effort to ensure agreement between nodes in the presence of floating
/// point / rounding error. This approach is still vulnerable when the true value is near the rounding boundary.
///
/// See: https://github.com/chainflip-io/chainflip-backend/issues/1803
async fn get_tracked_data<EthRpcClient: EthRpcApi + Send + Sync>(
    rpc: &EthRpcClient,
    block_number: u64,
    priority_fee_percentile: u8,
) -> anyhow::Result<TrackedData<Ethereum>> {
    let fee_history = rpc
        .fee_history(
            U256::one(),
            BlockNumber::Number(U64::from(block_number)),
            Some(vec![priority_fee_percentile as f64 / 100_f64]),
        )
        .await?;

    Ok(TrackedData::<Ethereum> {
        block_height: block_number,
        base_fee: context!(fee_history.base_fee_per_gas.first())?.as_u128(),
        priority_fee: context!(context!(context!(fee_history.reward)?.first())?.first())?.as_u128(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_tracked_data() {
        use crate::eth::rpc::MockEthRpcApi;

        const BLOCK_HEIGHT: u64 = 42;
        const BASE_FEE: u128 = 40_000_000_000;
        const PRIORITY_FEE: u128 = 5_000_000_000;

        let mut rpc = MockEthRpcApi::new();

        // ** Rpc Api Assumptions **
        rpc.expect_fee_history()
            .once()
            .returning(|_, block_number, _| {
                Ok(web3::types::FeeHistory {
                    oldest_block: block_number,
                    base_fee_per_gas: vec![U256::from(BASE_FEE)],
                    gas_used_ratio: vec![],
                    reward: Some(vec![vec![U256::from(PRIORITY_FEE)]]),
                })
            });
        // ** Rpc Api Assumptions **

        assert_eq!(
            get_tracked_data(&rpc, BLOCK_HEIGHT, 50).await.unwrap(),
            TrackedData {
                block_height: BLOCK_HEIGHT,
                base_fee: BASE_FEE,
                priority_fee: PRIORITY_FEE,
            }
        );
    }
}