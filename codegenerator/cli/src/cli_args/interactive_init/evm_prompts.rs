use super::{
    clap_definitions::evm::{
        ContractImportArgs, ExplorerImportArgs, LocalImportArgs, LocalOrExplorerImport,
    },
    prompt_abi_file_path, prompt_contract_address, prompt_contract_name, prompt_events_selection,
    validation::UniqueValueValidator,
    SelectItem,
};
use crate::{
    cli_args::interactive_init::validation::filter_duplicate_events,
    config_parsing::{
        chain_helpers::{HypersyncNetwork, Network, NetworkWithExplorer},
        contract_import::converters::{
            self, AutoConfigError, AutoConfigSelection, ContractImportNetworkSelection,
            ContractImportSelection,
        },
        human_config::evm::EventConfig,
    },
    evm::address::Address,
};
use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use inquire::{validator::Validation, CustomType, Select, Text};
use std::{env, path::PathBuf, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

fn prompt_abi_events_selection(events: Vec<ethers::abi::Event>) -> Result<Vec<ethers::abi::Event>> {
    prompt_events_selection(
        events
            .into_iter()
            .map(|abi_event| SelectItem {
                display: EventConfig::event_string_from_abi_event(&abi_event),
                item: abi_event,
            })
            .collect(),
    )
    .context("Failed selecting ABI events")
}

///Represents the choice a user makes for adding values to
///their auto config selection
#[derive(strum_macros::Display, EnumIter, Default, PartialEq)]
enum AddNewContractOption {
    #[default]
    #[strum(serialize = "I'm finished")]
    Finished,
    #[strum(serialize = "Add a new address for same contract on same network")]
    AddAddress,
    #[strum(serialize = "Add a new network for same contract")]
    AddNetwork,
    #[strum(serialize = "Add a new contract (with a different ABI)")]
    AddContract,
}

impl ContractImportNetworkSelection {
    ///Recursively asks to add an address to ContractImportNetworkSelection
    fn prompt_add_contract_address_to_network_selection(
        self,
        current_contract_name: &str,
        //Used in the case where we want to preselect add address
        preselected_add_new_contract_option: Option<AddNewContractOption>,
    ) -> Result<(Self, AddNewContractOption)> {
        let selected_option = match preselected_add_new_contract_option {
            Some(preselected) => preselected,
            None => {
                let options = AddNewContractOption::iter().collect::<Vec<_>>();
                let help_message = format!(
                    "Current contract: {}, on network: {}",
                    current_contract_name, self.network
                );
                Select::new("Would you like to add another contract?", options)
                    .with_starting_cursor(0)
                    .with_help_message(&help_message)
                    .prompt()
                    .context("Failed prompting for add contract")?
            }
        };

        if selected_option == AddNewContractOption::AddAddress {
            let address = prompt_contract_address(Some(&self.addresses))
                .context("Failed prompting user for new address")?;
            let updated_selection = self.add_address(address);

            updated_selection
                .prompt_add_contract_address_to_network_selection(current_contract_name, None)
        } else {
            Ok((self, selected_option))
        }
    }
}

impl ContractImportSelection {
    //Recursively asks to add networks with addresses to ContractImportNetworkSelection
    fn prompt_add_network_to_contract_import_selection(
        self,
        add_new_contract_option: AddNewContractOption,
    ) -> Result<(Self, AddNewContractOption)> {
        if add_new_contract_option == AddNewContractOption::AddNetwork {
            //In a new network case, no RPC url could be
            //derived from CLI flags
            const NO_RPC_URL: Option<String> = None;

            //Select a new network (not from the list of existing network ids already added)
            let selected_network = prompt_for_network_id(&NO_RPC_URL, self.get_network_ids())
                .context("Failed selecting network")?;

            //Instantiate a network_selection without any  contract addresses
            let network_selection =
                ContractImportNetworkSelection::new_without_addresses(selected_network);
            //Populate contract addresses with prompt
            let (network_selection, add_new_contract_option) = network_selection
                .prompt_add_contract_address_to_network_selection(
                    &self.name,
                    Some(AddNewContractOption::AddAddress),
                )
                .context("Failed adding new contract address")?;

            //Add the network to the contract selection
            let contract_selection = self.add_network(network_selection);

            //Reprompt to add more or exit
            contract_selection
                .prompt_add_network_to_contract_import_selection(add_new_contract_option)
        } else {
            //Exit if the user does not want to add more networks
            Ok((self, add_new_contract_option))
        }
    }
}

impl AutoConfigSelection {
    ///Recursively prompts to import a new contract or exits
    #[async_recursion]
    async fn prompt_for_add_contract_import_selection(
        self,
        add_new_contract_option: AddNewContractOption,
    ) -> Result<Self> {
        if add_new_contract_option == AddNewContractOption::AddContract {
            //Import a new contract
            let (contract_import_selection, add_new_contract_option) =
                ContractImportArgs::default()
                    .get_contract_import_selection()
                    .await
                    .context("Failed getting new contract import selection")?;

            //Add contract to AutoConfigSelection, method will handle duplicate names
            //and prompting for new names
            let auto_config_selection = self
                .add_contract_with_prompt(contract_import_selection)
                .context("Failed adding contract import selection to AutoConfigSelection")?;

            auto_config_selection
                .prompt_for_add_contract_import_selection(add_new_contract_option)
                .await
        } else {
            Ok(self)
        }
    }

    ///Calls add_contract but handles case where these is a name collision and prompts for a new
    ///name
    fn add_contract_with_prompt(
        self,
        contract_import_selection: ContractImportSelection,
    ) -> Result<Self> {
        self.add_contract(contract_import_selection)
            .or_else(|e| match e {
                AutoConfigError::ContractNameExists(mut contract, auto_config_selection) => {
                    let prompt_text = format!(
                        "Contract with name {} already exists in your project. Please provide an \
                         alternative name: ",
                        contract.name
                    );
                    contract.name = Text::new(&prompt_text)
                        .prompt()
                        .context("Failed prompting for new Contract name")?;
                    auto_config_selection.add_contract_with_prompt(contract)
                }
            })
    }
}

impl ContractImportArgs {
    ///Constructs AutoConfigSelection vial cli args and prompts
    pub async fn get_auto_config_selection(&self) -> Result<AutoConfigSelection> {
        let (contract_import_selection, add_new_contract_option) = self
            .get_contract_import_selection()
            .await
            .context("Failed getting ContractImportSelection")?;

        let auto_config_selection = AutoConfigSelection::new(contract_import_selection);

        let auto_config_selection = if !self.single_contract {
            auto_config_selection
                .prompt_for_add_contract_import_selection(add_new_contract_option)
                .await
                .context("Failed adding contracts to AutoConfigSelection")?
        } else {
            auto_config_selection
        };

        Ok(auto_config_selection)
    }

    ///Constructs ContractImportSelection via cli args and prompts
    async fn get_contract_import_selection(
        &self,
    ) -> Result<(ContractImportSelection, AddNewContractOption)> {
        //Construct ContractImportSelection via explorer or local import
        let (contract_import_selection, add_new_contract_option) =
            match &self.get_local_or_explorer_import()? {
                LocalOrExplorerImport::Explorer(explorer_import_args) => self
                    .get_contract_import_selection_from_explore_import_args(explorer_import_args)
                    .await
                    .context("Failed getting ContractImportSelection from explorer")?,
                LocalOrExplorerImport::Local(local_import_args) => self
                    .get_contract_import_selection_from_local_import_args(local_import_args)
                    .await
                    .context("Failed getting local contract selection")?,
            };

        //If --single-contract flag was not passed in, prompt to ask the user
        //if they would like to add networks to their contract selection
        let (contract_import_selection, add_new_contract_option) = if !self.single_contract {
            contract_import_selection
                .prompt_add_network_to_contract_import_selection(add_new_contract_option)
                .context("Failed adding networks to ContractImportSelection")?
        } else {
            (contract_import_selection, AddNewContractOption::Finished)
        };

        Ok((contract_import_selection, add_new_contract_option))
    }

    //Constructs ContractImportSelection via local prompt. Uses abis and manual
    //network/contract config
    async fn get_contract_import_selection_from_local_import_args(
        &self,
        local_import_args: &LocalImportArgs,
    ) -> Result<(ContractImportSelection, AddNewContractOption)> {
        let parsed_abi = local_import_args
            .get_abi()
            .context("Failed getting parsed abi")?;
        let mut abi_events: Vec<ethers::abi::Event> = parsed_abi.events().cloned().collect();

        if !self.all_events {
            abi_events = prompt_abi_events_selection(abi_events)?;
        }

        let network = local_import_args
            .get_network()
            .context("Failed getting chosen network")?;

        let contract_name = local_import_args
            .get_contract_name()
            .context("Failed getting contract name")?;

        let address = self
            .get_contract_address()
            .context("Failed getting contract address")?;

        let network_selection = ContractImportNetworkSelection::new(network, address);

        //If the flag for --single-contract was not added, continue to prompt for adding
        //addresses to the given network for this contract
        let (network_selection, add_new_contract_option) = if !self.single_contract {
            network_selection
                .prompt_add_contract_address_to_network_selection(&contract_name, None)
                .context("Failed prompting for more contract addresses on network")?
        } else {
            (network_selection, AddNewContractOption::Finished)
        };

        let contract_selection =
            ContractImportSelection::new(contract_name, network_selection, abi_events);

        Ok((contract_selection, add_new_contract_option))
    }

    ///Constructs ContractImportSelection via block explorer requests.
    async fn get_contract_import_selection_from_explore_import_args(
        &self,
        explorer_import_args: &ExplorerImportArgs,
    ) -> Result<(ContractImportSelection, AddNewContractOption)> {
        let network_with_explorer = explorer_import_args
            .get_network_with_explorer()
            .context("Failed getting NetworkWithExporer")?;

        let chosen_contract_address = self
            .get_contract_address()
            .context("Failed getting contract address")?;

        let contract_selection_from_etherscan = ContractImportSelection::from_etherscan(
            &network_with_explorer,
            chosen_contract_address,
        )
        .await
        .context("Failed getting ContractImportSelection from explorer")?;

        let ContractImportSelection {
            name,
            networks,
            events,
        } = if !self.all_events {
            let events = prompt_abi_events_selection(contract_selection_from_etherscan.events)?;
            ContractImportSelection {
                events,
                ..contract_selection_from_etherscan
            }
        } else {
            contract_selection_from_etherscan
        };

        let last_network_selection = networks.last().cloned().ok_or_else(|| {
            anyhow!("Expected a network seletion to be constructed with ContractImportSelection")
        })?;

        //If the flag for --single-contract was not added, continue to prompt for adding
        //addresses to the given network for this contract
        let (network_selection, add_new_contract_option) = if !self.single_contract {
            last_network_selection
                .prompt_add_contract_address_to_network_selection(&name, None)
                .context("Failed prompting for more contract addresses on network")?
        } else {
            (last_network_selection, AddNewContractOption::Finished)
        };

        let contract_selection = ContractImportSelection::new(name, network_selection, events);

        Ok((contract_selection, add_new_contract_option))
    }

    ///Takes either the address passed in by cli flag or prompts
    ///for an address
    fn get_contract_address(&self) -> Result<Address> {
        match &self.contract_address {
            Some(c) => Ok(c.clone()),
            None => prompt_contract_address(None),
        }
    }

    ///Takes either the "local" or "explorer" subcommand from the cli args
    ///or prompts for a choice from the user
    fn get_local_or_explorer_import(&self) -> Result<LocalOrExplorerImport> {
        match &self.local_or_explorer {
            Some(v) => Ok(v.clone()),
            None => {
                let options = LocalOrExplorerImport::iter().collect();

                Select::new(
                    "Would you like to import from a block explorer or a local abi?",
                    options,
                )
                .prompt()
                .context("Failed prompting for import from block explorer or local abi")
            }
        }
    }
}

///Prompts for a Supported network or for the user to enter an
///id, if it is unsupported it requires an RPC url. If the rpc is already
///known it can be passed in as the first arg. Otherwise this will be prompted.
///It also checks that the network does not belong to a given list of network ids
///To validate that a user is not double selecting a network id
fn prompt_for_network_id(
    opt_rpc_url: &Option<String>,
    already_selected_ids: Vec<u64>,
) -> Result<converters::NetworkKind> {
    //The first option of the list, funnels the user to enter a u64
    let enter_id = "<Enter Network Id>";

    //Select one of our supported networks
    let networks = HypersyncNetwork::iter()
        //Don't allow selection of networks that have been previously
        //selected.
        .filter(|n| {
            let network_id = *n as u64;
            !already_selected_ids.contains(&network_id)
        })
        .map(|n| n.to_string())
        .collect::<Vec<_>>();

    //User's options to either enter an id or select a supported network
    let options = vec![vec![enter_id.to_string()], networks].concat();

    //Action prompt
    let choose_from_networks = Select::new("Choose network:", options)
        .prompt()
        .context("Failed during prompt for abi file path")?;

    let selected = match choose_from_networks.as_str() {
        //If the user's choice evaluates to the enter network id option, prompt them for
        //a network id
        choice if choice == enter_id => {
            let network_id = CustomType::<u64>::new("Enter the network id:")
                //Validate that this ID is not already selected
                .with_validator(UniqueValueValidator::new(already_selected_ids))
                .with_error_message("Invalid network id input, please enter a number")
                .prompt()?;

            //Convert the id into a supported or unsupported network.
            //If unsupported, it will use the optional rpc url or prompt
            //for an rpc url
            get_converter_network_u64(network_id, opt_rpc_url)?
        }
        //If a supported network choice was selected. We should be able to
        //parse it back to a supported network since it was serialized as a
        //string
        choice => converters::NetworkKind::Supported(
            HypersyncNetwork::from_str(&choice)
                .context("Unexpected input, not a supported network.")?,
        ),
    };

    Ok(selected)
}

//Takes a u64 network ID and turns it into either "Supported" network or
//"Unsupported" where we need an RPC url. If the RPC url is known, pass it
//in as the 2nd arg otherwise prompt for an rpc url
fn get_converter_network_u64(
    network_id: u64,
    rpc_url: &Option<String>,
) -> Result<converters::NetworkKind> {
    let maybe_supported_network =
        Network::from_network_id(network_id).and_then(|n| Ok(HypersyncNetwork::try_from(n)?));

    let network = match maybe_supported_network {
        Ok(s) => converters::NetworkKind::Supported(s),
        Err(_) => {
            let rpc_url = match rpc_url {
                Some(r) => r.clone(),
                None => prompt_for_rpc_url()?,
            };
            converters::NetworkKind::Unsupported(network_id, rpc_url)
        }
    };

    Ok(network)
}

///Prompt the user to enter an rpc url
fn prompt_for_rpc_url() -> Result<String> {
    Text::new(
        "You have entered a network that is unsupported by our servers. Please provide an rpc url \
         (this can be edited later in config.yaml):",
    )
    .prompt()
    .context("Failed during rpc url prompt")
}

impl ExplorerImportArgs {
    ///Either take the NetworkWithExplorer value from the cli args or prompt
    ///for a user to select one.
    fn get_network_with_explorer(&self) -> Result<NetworkWithExplorer> {
        let chosen_network = match &self.blockchain {
            Some(chain) => chain.clone(),
            None => {
                let options = NetworkWithExplorer::iter()
                    //Filter only our supported networks
                    .filter(|&n| {
                        HypersyncNetwork::iter()
                            //able to cast as u64 because networks enum
                            //uses repr(u64) attribute
                            .find(|&sn| n as u64 == sn as u64)
                            .is_some()
                    })
                    .collect();

                Select::new(
                    "Which blockchain would you like to import a contract from?",
                    options,
                )
                .prompt()?
            }
        };

        Ok(chosen_network)
    }
}

impl LocalImportArgs {
    fn parse_contract_abi(abi_path: PathBuf) -> anyhow::Result<ethers::abi::Contract> {
        let abi_file = std::fs::read_to_string(&abi_path).context(format!(
            "Failed to read abi file at {:?}, relative to the current directory {:?}",
            abi_path,
            env::current_dir().unwrap_or(PathBuf::default())
        ))?;

        let abi: ethers::abi::Contract = serde_json::from_str(&abi_file).context(format!(
            "Failed to deserialize ABI at {:?} -  Please ensure the ABI file is formatted correctly \
            or contact the team.",
            abi_path
        ))?;

        Ok(abi)
    }

    ///Internal function to get the abi path from the cli args or prompt for
    ///a file path to the abi
    fn get_abi_path_string(&self) -> Result<String> {
        match &self.abi_file {
            Some(p) => Ok(p.clone()),
            None => prompt_abi_file_path(|path| {
                let maybe_parsed_abi = Self::parse_contract_abi(PathBuf::from(path));
                match maybe_parsed_abi {
                    Ok(_) => Validation::Valid,
                    Err(e) => Validation::Invalid(e.into()),
                }
            }),
        }
    }

    ///Get the file path for the abi and parse it into an abi
    fn get_abi(&self) -> Result<ethers::abi::Abi> {
        let abi_path_string = self.get_abi_path_string()?;

        let mut parsed_abi = Self::parse_contract_abi(PathBuf::from(abi_path_string))
            .context("Failed to parse abi")?;

        parsed_abi.events = filter_duplicate_events(parsed_abi.events);

        Ok(parsed_abi)
    }

    ///Gets the network from from cli args or prompts for
    ///a network
    fn get_network(&self) -> Result<converters::NetworkKind> {
        match &self.blockchain {
            Some(b) => {
                let network_id: u64 = (b.clone()).into();
                get_converter_network_u64(network_id, &self.rpc_url)
            }
            None => prompt_for_network_id(&self.rpc_url, vec![]),
        }
    }

    ///Prompts for a contract name
    fn get_contract_name(&self) -> Result<String> {
        match &self.contract_name {
            Some(n) => Ok(n.clone()),
            None => prompt_contract_name(),
        }
    }
}
