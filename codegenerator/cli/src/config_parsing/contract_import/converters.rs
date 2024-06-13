use super::etherscan_helpers::fetch_contract_auto_selection_from_etherscan;
use crate::{
    cli_args::init_config::Language,
    config_parsing::{
        chain_helpers::{HypersyncNetwork, NetworkWithExplorer},
        human_config::{
            evm::{ContractConfig, EventConfig, Network},
            GlobalContract, HumanConfig, NetworkContract, RpcConfig, SyncSourceConfig,
        },
    },
    evm::address::Address,
    init_config::InitConfig,
    utils::unique_hashmap,
};
use anyhow::{Context, Result};
use itertools::{self, Itertools};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};
use thiserror;

///A an object that holds all the values a user can select during
///the auto config generation. Values can come from etherscan or
///abis etc.
#[derive(Clone, Debug)]
pub struct AutoConfigSelection {
    selected_contracts: Vec<ContractImportSelection>,
}

#[derive(thiserror::Error, Debug)]
pub enum AutoConfigError {
    #[error("Contract '{}' already exists in AutoConfigSelection", .0.name)]
    ContractNameExists(ContractImportSelection, AutoConfigSelection),
}

impl AutoConfigSelection {
    pub fn new(selected_contract: ContractImportSelection) -> Self {
        Self {
            selected_contracts: vec![selected_contract],
        }
    }

    pub fn add_contract(
        mut self,
        contract: ContractImportSelection,
    ) -> Result<Self, AutoConfigError> {
        let contract_name_lower = contract.name.to_lowercase();
        let contract_name_exists = self
            .selected_contracts
            .iter()
            .find(|c| &c.name.to_lowercase() == &contract_name_lower)
            .is_some();

        if contract_name_exists {
            //TODO: Handle more cases gracefully like:
            // - contract + event is exact match, in which case it should just merge networks and
            // addresses
            // - Contract has some matching addresses to another contract but all different events
            // - Contract has some matching events as another contract?
            Err(AutoConfigError::ContractNameExists(contract, self))?
        } else {
            self.selected_contracts.push(contract);
            Ok(self)
        }
    }

    pub async fn from_etherscan(
        network: &NetworkWithExplorer,
        address: Address,
    ) -> anyhow::Result<Self> {
        let selected_contract = fetch_contract_auto_selection_from_etherscan(address, network)
            .await
            .context("Failed fetching selected contract")?;

        Ok(Self::new(selected_contract))
    }
}

///The hierarchy is based on how you would add items to
///your selection as you go. Ie. Once you have constructed
///the selection of a contract you can add more addresses or
///networks
#[derive(Clone, Debug)]
pub struct ContractImportSelection {
    pub name: String,
    pub networks: Vec<ContractImportNetworkSelection>,
    pub events: Vec<ethers::abi::Event>,
}

impl ContractImportSelection {
    pub fn new(
        name: String,
        network_selection: ContractImportNetworkSelection,
        events: Vec<ethers::abi::Event>,
    ) -> Self {
        Self {
            name,
            networks: vec![network_selection],
            events,
        }
    }

    pub fn add_network(mut self, network_selection: ContractImportNetworkSelection) -> Self {
        self.networks.push(network_selection);
        self
    }

    pub async fn from_etherscan(
        network: &NetworkWithExplorer,
        address: Address,
    ) -> anyhow::Result<Self> {
        fetch_contract_auto_selection_from_etherscan(address, network).await
    }

    pub fn get_network_ids(&self) -> Vec<u64> {
        self.networks
            .iter()
            .map(|n| n.network.get_network_id())
            .collect()
    }
}

type NetworkId = u64;
type RpcUrl = String;

#[derive(Clone, Debug)]
pub enum NetworkKind {
    Supported(HypersyncNetwork),
    Unsupported(NetworkId, RpcUrl),
}

impl NetworkKind {
    pub fn get_network_id(&self) -> NetworkId {
        match self {
            Self::Supported(n) => n.clone() as u64,
            Self::Unsupported(n, _) => *n,
        }
    }
}

impl Display for NetworkKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Supported(n) => write!(f, "{}", n),
            Self::Unsupported(n, _) => write!(f, "{}", n),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContractImportNetworkSelection {
    pub network: NetworkKind,
    pub addresses: Vec<Address>,
}

impl ContractImportNetworkSelection {
    pub fn new(network: NetworkKind, address: Address) -> Self {
        Self {
            network,
            addresses: vec![address],
        }
    }

    pub fn new_without_addresses(network: NetworkKind) -> Self {
        Self {
            network,
            addresses: vec![],
        }
    }

    pub fn add_address(mut self, address: Address) -> Self {
        self.addresses.push(address);

        self
    }
}

///Converts the selection object into a human config
type ContractName = String;
impl AutoConfigSelection {
    pub fn to_human_config(self: &Self, init_config: &InitConfig) -> Result<HumanConfig> {
        let mut networks_map: HashMap<u64, Network> = HashMap::new();
        let mut global_contracts: HashMap<ContractName, GlobalContract<ContractConfig>> =
            HashMap::new();

        for selected_contract in self.selected_contracts.clone() {
            let is_multi_chain_contract = selected_contract.networks.len() > 1;

            let events: Vec<EventConfig> = selected_contract
                .events
                .into_iter()
                .map(|event| EventConfig {
                    event: EventConfig::event_string_from_abi_event(&event),
                    required_entities: None,
                    is_async: None,
                })
                .collect();

            let handler = get_event_handler_directory(&init_config.language);

            let config = if is_multi_chain_contract {
                //Add the contract to global contract config and return none for local contract
                //config
                let global_contract = GlobalContract {
                    name: selected_contract.name.clone(),
                    config: ContractConfig {
                        abi_file_path: None,
                        handler,
                        events,
                    },
                };

                unique_hashmap::try_insert(
                    &mut global_contracts,
                    selected_contract.name.clone(),
                    global_contract,
                )
                .context(format!(
                    "Unexpected, failed to add global contract {}. Contract should have unique \
                     names",
                    selected_contract.name
                ))?;
                None
            } else {
                //Return some for local contract config
                Some(ContractConfig {
                    abi_file_path: None,
                    handler,
                    events,
                })
            };

            for selected_network in &selected_contract.networks {
                let address = selected_network
                    .addresses
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .into();

                let network = networks_map
                    .entry(selected_network.network.get_network_id())
                    .or_insert({
                        let sync_source = match &selected_network.network {
                            NetworkKind::Supported(_) => None,
                            NetworkKind::Unsupported(_, url) => {
                                Some(SyncSourceConfig::RpcConfig(RpcConfig {
                                    url: url.clone(),
                                    unstable__sync_config: None,
                                }))
                            }
                        };

                        Network {
                            id: selected_network.network.get_network_id(),
                            sync_source,
                            start_block: 0,
                            end_block: None,
                            confirmed_block_threshold: None,
                            contracts: Vec::new(),
                        }
                    });

                let contract = NetworkContract {
                    name: selected_contract.name.clone(),
                    address,
                    config: config.clone(),
                };

                network.contracts.push(contract);
            }
        }

        let contracts = match global_contracts
            .into_values()
            .sorted_by_key(|v| v.name.clone())
            .collect::<Vec<_>>()
        {
            values if values.is_empty() => None,
            values => Some(values),
        };

        Ok(HumanConfig {
            name: init_config.name.clone(),
            description: None,
            schema: None,
            contracts,
            networks: Some(networks_map.into_values().sorted_by_key(|v| v.id).collect()),
            unordered_multichain_mode: None,
            event_decoder: None,
            rollback_on_reorg: None,
            save_full_history: None,
            fuel: None,
        })
    }
}

// Logic to get the event handler directory based on the language
fn get_event_handler_directory(language: &Language) -> String {
    match language {
        Language::ReScript => "./src/EventHandlers.bs.js".to_string(),
        Language::TypeScript => "src/EventHandlers.ts".to_string(),
        Language::JavaScript => "./src/EventHandlers.js".to_string(),
    }
}
