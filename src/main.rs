mod exts;

use clap::Parser;
use reth::{
    cli::{
        config::RethRpcConfig,
        ext::{RethCliExt, RethNodeCommandConfig},
        Cli,
    },
    network::{NetworkInfo, Peers},
    providers::{
        BlockReaderIdExt, CanonStateSubscriptions, StateProviderFactory,
    },
    rpc::builder::{RethModuleRegistry, TransportRpcModules},
    tasks::TaskSpawner,
};
use reth_transaction_pool::TransactionPool;

use exts::{EthExt, EthExtApiServer};

#[derive(Debug, Clone, Copy, Default, clap::Args)]
struct RethCliExtentions {
    /// CLI flag to enable the eth extension namespace
    #[clap(long)]
    pub eth_ext: bool,
}


impl RethNodeCommandConfig for RethCliExtentions {
    // This is the entrypoint for the CLI to extend the RPC server with custom rpc namespaces.
    fn extend_rpc_modules<Conf, Provider, Pool, Network, Tasks, Events>(
        &mut self,
        _config: &Conf,
        registry: &mut RethModuleRegistry<Provider, Pool, Network, Tasks, Events>,
        modules: &mut TransportRpcModules,
    ) -> eyre::Result<()>
        where
            Conf: RethRpcConfig,
            Provider: BlockReaderIdExt + StateProviderFactory + Clone + Unpin + 'static,
            Pool: TransactionPool + Clone + 'static,
            Network: NetworkInfo + Peers + Clone + 'static,
            Tasks: TaskSpawner + Clone + 'static,
            Events: CanonStateSubscriptions + Clone + 'static,
    {
        if self.eth_ext {
            let provider = registry.provider().clone();
            let ext = EthExt::new(provider);
            modules.merge_configured(ext.into_rpc())?;
        }

        Ok(())
    }
}


/// The type that tells the reth CLI what extensions to use
struct ExtendedRethCli;

impl RethCliExt for ExtendedRethCli {
    /// This tells the reth CLI to install the extension namespace
    type Node = RethCliExtentions;
}

fn main() {
    Cli::<ExtendedRethCli>::parse().run().unwrap();
}
