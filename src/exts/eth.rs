use reth_primitives::{Address, BlockId, U256, H256, KECCAK_EMPTY};
use serde::{Deserialize, Serialize};
use jsonrpsee::{core::RpcResult, proc_macros::rpc, types::ErrorObjectOwned};
use reth::providers::{AccountReader, BlockReaderIdExt, StateProviderBox, StateProviderFactory};
use reth_rpc::eth::error::EthResult;
use reth_interfaces::Result;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
#[allow(missing_docs)]
pub struct AccountExt {
    pub balance: U256,
    pub nonce: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_hash: Option<H256>,
}

impl AccountExt {
    pub fn is_empty(&self) -> bool {
        let is_bytecode_empty = match self.code_hash {
            None => true,
            Some(hash) => hash == KECCAK_EMPTY,
        };

        self.nonce == 0 && self.balance == U256::ZERO && is_bytecode_empty
    }
}


/// Define extenstion namespace and methods
#[rpc(server, namespace = "eth")]
pub trait EthExtApi {
    /// Returns the number of transactions in the pool.
    #[method(name = "getAccountExt")]
    fn get_account_ext(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<AccountExt>;
}

pub struct EthExt<Provider> {
    inner: EthExtApiInner<Provider>,
}

/// Inner struct to wrap the provider, pool and other components all together
pub struct EthExtApiInner<Provider> {
    provider: Provider,
}

impl<Provider> EthExt<Provider>
    where Provider: BlockReaderIdExt + StateProviderFactory + Clone + 'static
{
    pub fn new(provider: Provider) -> Self {
        Self {
            inner: EthExtApiInner {
                provider,
            },
        }
    }

    pub fn provider(&self) -> &Provider {
        &self.inner.provider
    }

    /// Returns the state at the given [BlockId] enum.
    pub fn state_at_block_id(&self, at: BlockId) -> EthResult<StateProviderBox<'_>> {
        Ok(self.provider().state_by_block_id(at)?)
    }

    /// Returns the state at the given [BlockId] enum or the latest.
    ///
    /// Convenience function to interprets `None` as `BlockId::Number(BlockNumberOrTag::Latest)`
    pub fn state_at_block_id_or_latest(
        &self,
        block_id: Option<BlockId>,
    ) -> EthResult<StateProviderBox<'_>> {
        if let Some(block_id) = block_id {
            self.state_at_block_id(block_id)
        } else {
            Ok(self.latest_state()?)
        }
    }

    /// Returns the _latest_ state
    pub fn latest_state(&self) -> Result<StateProviderBox<'_>> {
        self.provider().latest()
    }
}

/// Implement the server side of these RPC methods for the extension namespace
impl<Provider> EthExtApiServer for EthExt<Provider>
    where Provider: BlockReaderIdExt + StateProviderFactory + Clone + 'static
{
    // Note: pending transactions are not taken into account here.
    fn get_account_ext(&self, address: Address, block_number: Option<BlockId>) -> RpcResult<AccountExt> {
        let state = self.state_at_block_id_or_latest(block_number)?;
        match state.basic_account(address) {
            Ok(Some(account)) => {
                let code_hash = match account.bytecode_hash {
                    None => None,
                    Some(hash) => {
                        if hash == KECCAK_EMPTY {
                            None
                        } else {
                            Some(hash)
                        }
                    }
                };

                Ok(AccountExt {
                    balance: account.balance,
                    nonce: account.nonce,
                    code_hash,
                })
            }
            Ok(None) => Ok(AccountExt {
                balance: U256::from(0),
                nonce: 0,
                code_hash: None,
            }),
            
            Err(e) => Err(ErrorObjectOwned::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                 e.to_string(),
                None::<()>,
            )),
        }
    }
}
