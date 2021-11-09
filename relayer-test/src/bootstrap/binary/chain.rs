/*!
    Helper functions for bootstrapping two relayer chain handles
    with connected foreign clients.
*/
use eyre::Report as Error;
use ibc_relayer::chain::handle::{ChainHandle, ProdChainHandle};
use ibc_relayer::config::{Config, SharedConfig};
use ibc_relayer::foreign_client::ForeignClient;
use ibc_relayer::registry::SharedRegistry;
use std::fs;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::info;

use crate::types::binary::chains::ConnectedChains;
use crate::types::config::TestConfig;
use crate::types::single::node::FullNode;
use crate::types::tagged::*;
use crate::types::wallet::{TestWallets, Wallet};
use crate::util::random::random_u32;

/**
   Bootstraps two relayer chain handles with connected foreign clients.

   Takes two [`FullNode`] values representing two different running
   full nodes, and return a [`ConnectedChains`] that contain the given
   full nodes together with the corresponding two [`ChainHandle`]s and
   [`ForeignClient`]s. Also accepts an [`FnOnce`] closure that modifies
   the relayer's [`Config`] before the chain handles are initialized.
*/
pub fn boostrap_chain_pair_with_nodes(
    test_config: &TestConfig,
    node_a: FullNode,
    node_b: FullNode,
    config_modifier: impl FnOnce(&mut Config),
) -> Result<ConnectedChains<impl ChainHandle, impl ChainHandle>, Error> {
    let mut config = Config::default();

    add_chain_config(&mut config, &node_a)?;
    add_chain_config(&mut config, &node_b)?;

    config_modifier(&mut config);

    let config_str = toml::to_string_pretty(&config)?;

    let config_path = test_config
        .chain_store_dir
        .join(format!("config-{:x}.toml", random_u32()));

    fs::write(&config_path, &config_str)?;

    info!(
        "written hermes config.toml to {}:\n{}",
        config_path.display(),
        config_str
    );

    let config = Arc::new(RwLock::new(config));

    let registry = new_registry(config.clone());

    // Pass in unique closure expressions `||{}` as the first argument so that
    // the returned chains are considered different types by Rust.
    // See [`spawn_chain_handle`] for more details.
    let handle_a = spawn_chain_handle(|| {}, &registry, &node_a)?;
    let handle_b = spawn_chain_handle(|| {}, &registry, &node_b)?;

    let client_a_to_b = ForeignClient::new(handle_b.clone(), handle_a.clone())?;
    let client_b_to_a = ForeignClient::new(handle_a.clone(), handle_b.clone())?;

    Ok(ConnectedChains::new(
        config_path,
        config,
        registry,
        handle_a,
        handle_b,
        MonoTagged::new(node_a),
        MonoTagged::new(node_b),
        client_a_to_b,
        client_b_to_a,
    ))
}

/**
   Spawn a new chain handle using the given [`SharedRegistry`] and
   [`FullNode`].

   The function accepts a proxy type `Seed` that should be unique
   accross multiple calls so that the returned [`ChainHandle`]
   have a unique type.

   For example, the following test should fail to compile:

   ```rust,compile_fail
   # use ibc_relayer_test::bootstrap::binary::chain::spawn_chain_handle;
   fn same<T>(_: T, _: T) {}

   let chain_a = spawn_chain_handle(|| {}, todo!(), todo!()).unwrap();
   let chain_b = spawn_chain_handle(|| {}, todo!(), todo!()).unwrap();
   same(chain_a, chain_b); // error: chain_a and chain_b have different types
   ```

   The reason is that Rust would give each closure expression `||{}` a
   [unique anonymous type](https://doc.rust-lang.org/reference/types/closure.html).
   When we instantiate two chains with different closure types,
   the resulting values would be considered by Rust to have different types.

   With this we can treat `chain_a` and `chain_b` having different types
   so that we do not accidentally mix them up later in the code.
*/
pub fn spawn_chain_handle<Seed>(
    _: Seed,
    registry: &SharedRegistry<impl ChainHandle + 'static>,
    node: &FullNode,
) -> Result<impl ChainHandle, Error> {
    let chain_id = &node.chain_driver.chain_id;
    let handle = registry.get_or_spawn(chain_id)?;

    add_keys_to_chain_handle(&handle, &node.wallets)?;

    Ok(handle)
}

fn add_key_to_chain_handle<Chain: ChainHandle>(
    chain: &Chain,
    wallet: &Wallet,
) -> Result<(), Error> {
    chain.add_key(wallet.id.0.clone(), wallet.key.clone())?;

    Ok(())
}

fn add_keys_to_chain_handle<Chain: ChainHandle>(
    chain: &Chain,
    wallets: &TestWallets,
) -> Result<(), Error> {
    add_key_to_chain_handle(chain, &wallets.relayer)?;
    add_key_to_chain_handle(chain, &wallets.user1)?;
    add_key_to_chain_handle(chain, &wallets.user2)?;

    Ok(())
}

fn new_registry(config: SharedConfig) -> SharedRegistry<ProdChainHandle> {
    <SharedRegistry<ProdChainHandle>>::new(config)
}

fn add_chain_config(config: &mut Config, running_node: &FullNode) -> Result<(), Error> {
    let chain_config = running_node.generate_chain_config()?;

    config.chains.push(chain_config);
    Ok(())
}
