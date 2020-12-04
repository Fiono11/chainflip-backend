use crate::vault::transactions::TransactionProvider;
use parking_lot::RwLock;
use std::sync::Arc;

use super::SideChainTx;

/// Interface to the substrate node
#[async_trait]
pub trait SubstrateNodeI {
    /// Submit transactions to the node's mempool
    fn submit_txs(&self, txs: Vec<SideChainTx>);
}

/// Real connection to the local Substrate Node (TODO)
pub struct SubstrateNode {}

impl SubstrateNode {
    /// Create default (TODO)
    pub fn new() -> Self {
        SubstrateNode {}
    }
}

#[async_trait]
impl SubstrateNodeI for SubstrateNode {
    fn submit_txs(&self, _txs: Vec<SideChainTx>) {
        info!("TODO: send transacitons to substrate");
    }
}

/// Test double for Substrate Node that always writes to the
/// provider as if transactions are immidiately added a
/// finalized block
pub struct FakeSubstrateNode<T>
where
    T: TransactionProvider,
{
    provider: Arc<RwLock<T>>,
}

impl<T: TransactionProvider> FakeSubstrateNode<T> {
    /// Construct an instance given transaction provider
    pub fn new(provider: Arc<RwLock<T>>) -> Self {
        FakeSubstrateNode { provider }
    }
}

#[async_trait]
impl<T> SubstrateNodeI for FakeSubstrateNode<T>
where
    T: TransactionProvider,
{
    fn submit_txs(&self, txs: Vec<SideChainTx>) {
        self.provider
            .write()
            .add_transactions(txs)
            .expect("Could not save txs");
    }
}
