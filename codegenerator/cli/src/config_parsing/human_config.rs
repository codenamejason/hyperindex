use super::validation;
use crate::{constants::links, utils::normalized_list::NormalizedList};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

type NetworkId = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct HumanConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<Vec<GlobalContractConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub networks: Vec<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unordered_multichain_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_decoder: Option<EventDecoder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_on_reorg: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_full_history: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum EventDecoder {
    Viem,
    HypersyncClient,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GlobalContractConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi_file_path: Option<String>,
    pub handler: String,
    pub events: Vec<ConfigEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HypersyncConfig {
    #[serde(alias = "url")]
    pub endpoint_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SyncSourceConfig {
    RpcConfig(RpcConfig),
    HypersyncConfig(HypersyncConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Network {
    pub id: NetworkId,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub sync_source: Option<SyncSourceConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmed_block_threshold: Option<i32>,
    pub start_block: i32,
    pub end_block: Option<i32>,
    pub contracts: Vec<NetworkContractConfig>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SyncConfigUnstable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_block_interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_multiplicative: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acceleration_additive: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval_ceiling: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_millis: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_timeout_millis: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[allow(non_snake_case)] //Stop compiler warning for the double underscore in unstable__sync_config
pub struct RpcConfig {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unstable__sync_config: Option<SyncConfigUnstable>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NetworkContractConfig {
    pub name: String,
    pub address: NormalizedList<String>,
    #[serde(flatten)]
    //If this is "None" it should be expected that
    //there is a global config for the contract
    pub local_contract_config: Option<LocalContractConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LocalContractConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi_file_path: Option<String>,
    pub handler: String,
    pub events: Vec<ConfigEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigEvent {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,
}

impl ConfigEvent {
    pub fn event_string_from_abi_event(abi_event: &ethers::abi::Event) -> String {
        format!(
            "{}({}){}",
            abi_event.name,
            abi_event
                .inputs
                .iter()
                .map(|input| {
                    let param_type = input.kind.to_string();
                    let indexed_keyword = if input.indexed { " indexed " } else { " " };
                    let param_name = input.name.clone();

                    format!("{}{}{}", param_type, indexed_keyword, param_name)
                })
                .collect::<Vec<_>>()
                .join(", "),
            if abi_event.anonymous {
                " anonymous"
            } else {
                ""
            },
        )
    }
}

fn strip_to_letters(string: &str) -> String {
    let mut pg_friendly_name = String::new();
    for c in string.chars() {
        if c.is_alphabetic() {
            pg_friendly_name.push(c);
        }
    }
    pg_friendly_name
}

pub fn deserialize_config_from_yaml(config_path: &PathBuf) -> anyhow::Result<HumanConfig> {
    let config = std::fs::read_to_string(config_path).context(format!(
        "EE104: Failed to resolve config path {0}. Make sure you're in the correct directory and \
         that a config file with the name {0} exists",
        &config_path
            .to_str()
            .unwrap_or("unknown config file name path"),
    ))?;

    let mut deserialized_yaml: HumanConfig = serde_yaml::from_str(&config).context(format!(
        "EE105: Failed to deserialize config. Visit the docs for more information {}",
        links::DOC_CONFIGURATION_FILE
    ))?;

    deserialized_yaml.name = strip_to_letters(&deserialized_yaml.name);

    // Validating the config file
    validation::validate_deserialized_config_yaml(config_path, &deserialized_yaml)?;

    Ok(deserialized_yaml)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config_parsing::human_config::EventDecoder;

    use super::{HumanConfig, LocalContractConfig, Network, NetworkContractConfig, NormalizedList};
    use serde_json::json;

    #[test]
    fn test_flatten_deserialize_local_contract() {
        let yaml = r#"
name: Contract1
handler: ./src/EventHandler.js
address: ["0x2E645469f354BB4F5c8a05B3b30A929361cf77eC"]
events: []
    "#;

        let deserialized: NetworkContractConfig = serde_yaml::from_str(yaml).unwrap();
        let expected = NetworkContractConfig {
            name: "Contract1".to_string(),
            address: NormalizedList::from(vec![
                "0x2E645469f354BB4F5c8a05B3b30A929361cf77eC".to_string()
            ]),
            local_contract_config: Some(LocalContractConfig {
                abi_file_path: None,
                handler: "./src/EventHandler.js".to_string(),
                events: vec![],
            }),
        };

        assert_eq!(expected, deserialized);
    }

    #[test]
    fn test_flatten_deserialize_local_contract_with_no_address() {
        let yaml = r#"
name: Contract1
handler: ./src/EventHandler.js
events: []
    "#;

        let deserialized: NetworkContractConfig = serde_yaml::from_str(yaml).unwrap();
        let expected = NetworkContractConfig {
            name: "Contract1".to_string(),
            address: vec![].into(),
            local_contract_config: Some(LocalContractConfig {
                abi_file_path: None,
                handler: "./src/EventHandler.js".to_string(),
                events: vec![],
            }),
        };

        assert_eq!(expected, deserialized);
    }

    #[test]
    fn test_flatten_deserialize_local_contract_with_single_address() {
        let yaml = r#"
name: Contract1
handler: ./src/EventHandler.js
address: "0x2E645469f354BB4F5c8a05B3b30A929361cf77eC"
events: []
    "#;

        let deserialized: NetworkContractConfig = serde_yaml::from_str(yaml).unwrap();
        let expected = NetworkContractConfig {
            name: "Contract1".to_string(),
            address: vec!["0x2E645469f354BB4F5c8a05B3b30A929361cf77eC".to_string()].into(),
            local_contract_config: Some(LocalContractConfig {
                abi_file_path: None,
                handler: "./src/EventHandler.js".to_string(),
                events: vec![],
            }),
        };

        assert_eq!(expected, deserialized);
    }

    #[test]
    fn test_flatten_deserialize_global_contract() {
        let yaml = r#"
name: Contract1
address: ["0x2E645469f354BB4F5c8a05B3b30A929361cf77eC"]
    "#;

        let deserialized: NetworkContractConfig = serde_yaml::from_str(yaml).unwrap();
        let expected = NetworkContractConfig {
            name: "Contract1".to_string(),
            address: NormalizedList::from(vec![
                "0x2E645469f354BB4F5c8a05B3b30A929361cf77eC".to_string()
            ]),
            local_contract_config: None,
        };

        assert_eq!(expected, deserialized);
    }

    #[test]
    fn deserialize_address() {
        let no_address = r#"null"#;
        let deserialized: NormalizedList<String> = serde_json::from_str(no_address).unwrap();
        assert_eq!(deserialized, NormalizedList::from(vec![]));

        let single_address = r#""0x123""#;
        let deserialized: NormalizedList<String> = serde_json::from_str(single_address).unwrap();
        assert_eq!(
            deserialized,
            NormalizedList::from(vec!["0x123".to_string()])
        );

        let multi_address = r#"["0x123", "0x456"]"#;
        let deserialized: NormalizedList<String> = serde_json::from_str(multi_address).unwrap();
        assert_eq!(
            deserialized,
            NormalizedList::from(vec!["0x123".to_string(), "0x456".to_string()])
        );
    }

    #[test]
    fn valid_name_conversion() {
        let name_with_space = super::strip_to_letters("My too lit to quit indexer");
        let expected_name_with_space = "Mytoolittoquitindexer";
        let name_with_special_chars = super::strip_to_letters("Myto@littoq$itindexer");
        let expected_name_with_special_chars = "Mytolittoqitindexer";
        let name_with_numbers = super::strip_to_letters("yes0123456789okay");
        let expected_name_with_numbers = "yesokay";
        assert_eq!(name_with_space, expected_name_with_space);
        assert_eq!(name_with_special_chars, expected_name_with_special_chars);
        assert_eq!(name_with_numbers, expected_name_with_numbers);
    }

    #[test]
    fn deserializes_factory_contract_config() {
        let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test/configs/factory-contract-config.yaml");

        let file_str = std::fs::read_to_string(config_path).unwrap();

        let cfg: HumanConfig = serde_yaml::from_str(&file_str).unwrap();

        println!("{:?}", cfg.networks[0].contracts[0]);

        assert!(cfg.networks[0].contracts[0].local_contract_config.is_some());
        assert!(cfg.networks[0].contracts[1].local_contract_config.is_some());
        assert_eq!(cfg.networks[0].contracts[1].address, None.into());
    }

    #[test]
    fn deserializes_dynamic_contract_config() {
        let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test/configs/dynamic-address-config.yaml");

        let file_str = std::fs::read_to_string(config_path).unwrap();

        let cfg: HumanConfig = serde_yaml::from_str(&file_str).unwrap();

        assert!(cfg.networks[0].contracts[0].local_contract_config.is_some());
        assert!(cfg.networks[1].contracts[0].local_contract_config.is_none());
    }

    #[test]
    fn deserializes_event_decoder() {
        assert_eq!(
            serde_json::from_value::<EventDecoder>(json!("viem")).unwrap(),
            EventDecoder::Viem
        );
        assert_eq!(
            serde_json::from_value::<EventDecoder>(json!("hypersync-client")).unwrap(),
            EventDecoder::HypersyncClient
        );
        assert_eq!(
            serde_json::to_value(&EventDecoder::HypersyncClient).unwrap(),
            json!("hypersync-client")
        );
        assert_eq!(
            serde_json::to_value(&EventDecoder::Viem).unwrap(),
            json!("viem")
        );
    }

    #[test]
    fn deserialize_underscores_between_numbers() {
        let num = serde_json::json!(2_000_000);
        let de: i32 = serde_json::from_value(num).unwrap();
        assert_eq!(2_000_000, de);
    }

    #[test]
    fn deserialize_network_with_underscores_between_numbers() {
        let network_json = serde_json::json!({"id": 1, "start_block": 2_000, "end_block": 2_000_000, "contracts": []});
        let de: Network = serde_json::from_value(network_json).unwrap();

        assert_eq!(
            Network {
                id: 1,
                sync_source: None,
                start_block: 2_000,
                confirmed_block_threshold: None,
                end_block: Some(2_000_000),
                contracts: vec![]
            },
            de
        );
    }
}
