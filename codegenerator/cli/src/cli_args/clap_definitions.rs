use crate::constants::project_paths::{DEFAULT_CONFIG_PATH, DEFAULT_GENERATED_PATH};

use clap::{Args, Parser, Subcommand};
use strum::{Display, EnumIter, EnumString};
use subenum::subenum;

use super::init_config::{self};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CommandLineArgs {
    #[clap(subcommand)]
    pub command: CommandType,
    #[command(flatten)]
    pub project_paths: ProjectPaths,
}

#[derive(Args, Debug, Clone)]
pub struct ProjectPaths {
    ///The directory of the project. Defaults to current dir ("./")
    #[arg(global = true, short, long)]
    pub directory: Option<String>,

    ///The directory within the project that generated code should output to
    #[arg(global = true, short, long, default_value_t=String::from(DEFAULT_GENERATED_PATH))]
    pub output_directory: String,

    ///The file in the project containing config.
    #[arg(global = true, long, default_value_t=String::from(DEFAULT_CONFIG_PATH))]
    pub config: String,
}

#[derive(Debug, Subcommand)]
pub enum CommandType {
    ///Initialize an indexer with one of the initialization options
    Init(InitArgs),

    /// Development commands for starting, stopping, and restarting the indexer with automatic codegen for any changed files
    Dev,

    /// Stop the local environment - delete the database and stop all processes (including Docker) for the current directory
    Stop,

    ///Generate indexing code from user-defined configuration & schema files
    Codegen,

    ///Prepare local environment for envio testing
    // #[clap(hide = true)]
    #[command(subcommand)]
    Local(LocalCommandTypes),

    ///Start the indexer without any automatic codegen
    Start(StartArgs),

    ///Print help into a markdown file
    ///Command to run: cargo run --bin envio -- print-all-help > CommandLineHelp.md
    #[clap(hide = true)]
    PrintAllHelp,
}

#[derive(Debug, Args)]
pub struct StartArgs {
    ///Clear your database and restart indexing from scratch
    #[arg(short = 'r', long, action)]
    pub restart: bool,
}

#[derive(Debug, Subcommand)]
pub enum LocalCommandTypes {
    /// Local Envio and ganache environment commands
    #[command(subcommand)]
    Docker(LocalDockerSubcommands),
    /// Local Envio database commands
    #[command(subcommand)]
    DbMigrate(DbMigrateSubcommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum LocalDockerSubcommands {
    ///Create docker images required for local environment
    Up,
    ///Delete existing docker images on local environment
    Down,
}

#[derive(Subcommand, Debug)]
pub enum DbMigrateSubcommands {
    ///Migrate latest schema to database
    Up,
    ///Drop database schema
    Down,
    ///Setup database by dropping schema and then running migrations
    Setup,
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    ///The name of your project
    #[arg(global = true, short, long)]
    pub name: Option<String>,

    ///Initialization option for creating an indexer
    #[command(subcommand)]
    pub init_commands: Option<InitFlow>,

    ///The language used to write handlers
    #[arg(global = true, short = 'l', long = "language")]
    #[clap(value_enum)]
    pub language: Option<init_config::Language>,
}

#[subenum(EvmInitFlowInteractive)]
#[derive(Subcommand, Debug, EnumIter, Display, EnumString, Clone)]
pub enum InitFlow {
    ///Initialize Evm indexer from an example template
    #[subenum(EvmInitFlowInteractive)]
    Template(evm::TemplateArgs),
    ///Initialize Evm indexer by importing config from a contract for a given chain
    #[subenum(EvmInitFlowInteractive)]
    #[strum(serialize = "Contract Import")]
    ContractImport(evm::ContractImportArgs),
    ///Initialize Evm indexer by migrating config from an existing subgraph
    #[clap(hide = true)] //hiding for now until this is more stable
    #[strum(serialize = "Subgraph Migration (Experimental)")]
    SubgraphMigration(evm::SubgraphMigrationArgs),
    ///Initialization option for creating Fuel indexer
    Fuel {
        #[command(subcommand)]
        init_flow: Option<fuel::InitFlow>,
    },
}

pub mod evm {
    use crate::{
        config_parsing::chain_helpers::{Network, NetworkWithExplorer},
        evm, init_config,
    };

    use anyhow::Context;
    use clap::{Args, Subcommand};
    use std::str::FromStr;
    use strum::{Display, EnumIter, EnumString};

    #[derive(Args, Debug, Default, Clone)]
    pub struct ContractImportArgs {
        ///Choose to import a contract from a local abi or
        ///using get values from an explorer using a contract address
        #[command(subcommand)]
        pub local_or_explorer: Option<LocalOrExplorerImport>,

        ///Contract address to generate the config from
        #[arg(global = true, short, long)]
        pub contract_address: Option<evm::address::Address>,

        ///If selected, prompt will not ask for additional contracts/addresses/networks
        #[arg(long, action)]
        pub single_contract: bool,

        ///If selected, prompt will not ask to confirm selection of events on a contract
        #[arg(long, action)]
        pub all_events: bool,
    }

    #[derive(Args, Debug, Default, Clone)]
    pub struct TemplateArgs {
        ///Name of the template to be used in initialization
        #[arg(short, long)]
        #[clap(value_enum)]
        pub template: Option<init_config::evm::Template>,
    }

    type SubgraphMigrationID = String;
    #[derive(Args, Debug, Default, Clone)]
    pub struct SubgraphMigrationArgs {
        ///Subgraph ID to start a migration from
        #[arg(short, long)]
        pub subgraph_id: Option<SubgraphMigrationID>,
    }

    #[derive(Subcommand, Debug, EnumIter, EnumString, Display, Clone)]
    pub enum LocalOrExplorerImport {
        ///Initialize by pulling the contract ABI from a block explorer
        #[strum(serialize = "Block Explorer")]
        Explorer(ExplorerImportArgs),
        ///Initialize from a local json ABI file
        #[strum(serialize = "Local ABI")]
        Local(LocalImportArgs),
    }

    #[derive(Args, Debug, Default, Clone)]
    pub struct ExplorerImportArgs {
        ///Network from which contract address should be fetched for migration
        #[arg(short, long)]
        pub blockchain: Option<NetworkWithExplorer>,
    }

    #[derive(Debug, Clone)]
    pub enum NetworkOrChainId {
        NetworkName(Network),
        ChainId(u64),
    }

    impl From<NetworkOrChainId> for u64 {
        fn from(value: NetworkOrChainId) -> Self {
            match value {
                NetworkOrChainId::ChainId(val) => val,
                NetworkOrChainId::NetworkName(name) => name as u64,
            }
        }
    }

    impl FromStr for NetworkOrChainId {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let res_network: Result<Network, _> = s.parse();

            match res_network {
                Ok(n) => Ok(NetworkOrChainId::NetworkName(n)),
                Err(_) => {
                    let chain_id: u64 = s.parse().context("Invalid network name or id")?;
                    Ok(NetworkOrChainId::ChainId(chain_id))
                }
            }
        }
    }

    #[derive(Args, Debug, Default, Clone)]
    pub struct LocalImportArgs {
        ///The path to a json abi file
        #[arg(short, long)]
        pub abi_file: Option<String>,
        ///The name of the contract
        #[arg(long)]
        pub contract_name: Option<String>,

        ///Network from which contract address should be fetched for migration
        #[arg(short, long)]
        pub blockchain: Option<NetworkOrChainId>,

        ///The rpc url to use if the network id used is unsupported by our hypersync
        #[arg(short, long)]
        pub rpc_url: Option<String>,
    }
}

pub mod fuel {
    use clap::{Args, Subcommand};
    use strum::{Display, EnumIter, EnumString};

    use crate::{fuel, init_config};

    #[derive(Subcommand, Debug, EnumIter, Display, EnumString, Clone)]
    pub enum InitFlow {
        ///Initialize Fuel indexer from an example template
        Template(TemplateArgs),
        ///Initialize Fuel indexer by importing config from a contract for a given chain
        #[strum(serialize = "Contract Import")]
        ContractImport(ContractImportArgs),
    }

    #[derive(Args, Debug, Default, Clone)]
    pub struct ContractImportArgs {
        ///Choose to import a contract from a local abi or
        ///using get values from an explorer using a contract address
        #[command(subcommand)]
        pub local_or_explorer: Option<LocalOrExplorerImport>,

        ///Contract address to generate the config from
        #[arg(global = true, short, long)]
        pub contract_address: Option<fuel::address::Address>,

        ///If selected, prompt will not ask for additional contracts/addresses/networks
        #[arg(long, action)]
        pub single_contract: bool,

        ///If selected, prompt will not ask to confirm selection of events on a contract
        #[arg(long, action)]
        pub all_events: bool,
    }

    #[derive(Subcommand, Debug, EnumIter, EnumString, Display, Clone)]
    pub enum LocalOrExplorerImport {
        // Not supported https://forum.fuel.network/t/get-abi-by-contract-address/5535
        // ///Initialize by pulling the contract ABI from a block explorer
        // #[strum(serialize = "Block Explorer")]
        // Explorer(ExplorerImportArgs),
        // ----
        ///Initialize from a local json ABI file
        #[strum(serialize = "Local ABI")]
        Local(LocalImportArgs),
    }

    #[derive(Args, Debug, Default, Clone)]
    pub struct LocalImportArgs {
        ///The path to a json abi file
        #[arg(short, long)]
        pub abi_file: Option<String>,
        ///The name of the contract
        #[arg(long)]
        pub contract_name: Option<String>,
    }

    #[derive(Args, Debug, Default, Clone)]
    pub struct TemplateArgs {
        ///Name of the template to be used in initialization
        #[arg(short, long)]
        #[clap(value_enum)]
        pub template: Option<init_config::fuel::Template>,
    }
}
