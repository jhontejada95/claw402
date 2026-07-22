//! ZeroClaw tool plugin for Claw402 procurement policy.

pub mod bazaar;
pub mod policy;

#[cfg(target_family = "wasm")]
mod component {
    wit_bindgen::generate!({
        path: "../../wit/v0",
        world: "tool-plugin",
        features: ["plugins-wit-v0"],
    });

    use std::collections::HashMap;

    use crate::bazaar::{rank_resources, BazaarResource, BazaarSearchResponse};
    use crate::policy::{evaluate_offer, PaymentOffer, PolicyConfig};
    use exports::zeroclaw::plugin::plugin_info::Guest as PluginInfo;
    use exports::zeroclaw::plugin::tool::{Guest as Tool, ToolResult};
    use zeroclaw::plugin::logging::{
        log_record, LogLevel, PluginAction, PluginEvent, PluginOutcome,
    };

    struct Claw402Policy;

    const PLUGIN_NAME: &str = "claw402-policy";
    const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
    const TOOL_NAME: &str = "claw402_policy";

    #[derive(serde::Deserialize)]
    #[serde(tag = "action", rename_all = "snake_case")]
    enum ExecuteArgs {
        InspectOffer {
            offer: PaymentOffer,
            #[serde(rename = "__config", default)]
            config: HashMap<String, String>,
        },
        RankResources {
            resources: Vec<BazaarResource>,
            #[serde(rename = "__config", default)]
            config: HashMap<String, String>,
        },
        Discover {
            query: String,
            #[serde(default = "default_limit")]
            limit: u8,
            #[serde(rename = "__config", default)]
            config: HashMap<String, String>,
        },
    }

    impl PluginInfo for Claw402Policy {
        fn plugin_name() -> String {
            PLUGIN_NAME.to_string()
        }

        fn plugin_version() -> String {
            PLUGIN_VERSION.to_string()
        }
    }

    impl Tool for Claw402Policy {
        fn name() -> String {
            TOOL_NAME.to_string()
        }

        fn description() -> String {
            "Inspect or rank x402 v2 Solana payment offers using operator-owned hard policy. \
             This tool is fail-closed: it never changes policy from model input and never signs. \
             Use inspect_offer before proposing a purchase and rank_resources for Bazaar results."
                .to_string()
        }

        fn parameters_schema() -> String {
            serde_json::json!({
                "type": "object",
                "oneOf": [
                    {
                        "properties": {
                            "action": {"const": "inspect_offer"},
                            "offer": {"type": "object"}
                        },
                        "required": ["action", "offer"]
                    },
                    {
                        "properties": {
                            "action": {"const": "rank_resources"},
                            "resources": {"type": "array", "items": {"type": "object"}}
                        },
                        "required": ["action", "resources"]
                    },
                    {
                        "properties": {
                            "action": {"const": "discover"},
                            "query": {"type": "string", "minLength": 1, "maxLength": 200},
                            "limit": {"type": "integer", "minimum": 1, "maximum": 20}
                        },
                        "required": ["action", "query"]
                    }
                ]
            })
            .to_string()
        }

        fn execute(args: String) -> Result<ToolResult, String> {
            let parsed: ExecuteArgs = match serde_json::from_str(&args) {
                Ok(value) => value,
                Err(error) => return failure(format!("invalid arguments: {error}")),
            };

            let output = match parsed {
                ExecuteArgs::InspectOffer { offer, config } => {
                    let policy = PolicyConfig::from_section(&config);
                    serde_json::to_string(&evaluate_offer(&offer, &policy))
                }
                ExecuteArgs::RankResources { resources, config } => {
                    let policy = PolicyConfig::from_section(&config);
                    serde_json::to_string(&rank_resources(&resources, &policy))
                }
                ExecuteArgs::Discover {
                    query,
                    limit,
                    config,
                } => discover(&query, limit, &config),
            };

            match output {
                Ok(output) => {
                    emit(
                        PluginAction::Validate,
                        PluginOutcome::Success,
                        "policy evaluated",
                    );
                    Ok(ToolResult {
                        success: true,
                        output,
                        error: None,
                    })
                }
                Err(error) => failure(format!("serialization failed: {error}")),
            }
        }
    }

    fn default_limit() -> u8 {
        10
    }

    fn discover(
        query: &str,
        limit: u8,
        config: &HashMap<String, String>,
    ) -> Result<String, serde_json::Error> {
        if query.is_empty() || query.len() > 200 {
            return serde_json::to_string(&serde_json::json!({
                "error": "query length must be between 1 and 200 bytes"
            }));
        }

        let policy = PolicyConfig::from_section(config);
        let Some(network) = policy.allowed_networks.iter().min() else {
            return serde_json::to_string(&serde_json::json!({
                "error": "no allowed network is configured"
            }));
        };
        let base = config
            .get("bazaar_search_url")
            .map(String::as_str)
            .unwrap_or("https://api.cdp.coinbase.com/platform/v2/x402/discovery/search");
        let Ok(mut url) = url::Url::parse(base) else {
            return serde_json::to_string(&serde_json::json!({
                "error": "configured Bazaar search URL is invalid"
            }));
        };
        if url.scheme() != "https" {
            return serde_json::to_string(&serde_json::json!({
                "error": "Bazaar search URL must use HTTPS"
            }));
        }
        url.query_pairs_mut()
            .append_pair("query", query)
            .append_pair("network", network)
            .append_pair("limit", &limit.clamp(1, 20).to_string());

        let response = match waki::Client::new()
            .get(url.as_str())
            .connect_timeout(std::time::Duration::from_secs(8))
            .send()
        {
            Ok(response) => response,
            Err(error) => {
                return serde_json::to_string(&serde_json::json!({
                    "error": format!("Bazaar request failed: {error}")
                }))
            }
        };
        let search: BazaarSearchResponse = match response.json() {
            Ok(search) => search,
            Err(error) => {
                return serde_json::to_string(&serde_json::json!({
                    "error": format!("Bazaar returned invalid JSON: {error}")
                }))
            }
        };
        serde_json::to_string(&serde_json::json!({
            "query": query,
            "partialResults": search.partial_results,
            "searchMethod": search.search_method,
            "eligible": rank_resources(&search.resources, &policy),
            "resourcesScreened": search.resources.len()
        }))
    }

    fn failure(message: String) -> Result<ToolResult, String> {
        emit(
            PluginAction::Fail,
            PluginOutcome::Failure,
            "policy evaluation failed",
        );
        Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some(message),
        })
    }

    fn emit(action: PluginAction, outcome: PluginOutcome, message: &str) {
        log_record(
            LogLevel::Info,
            &PluginEvent {
                function_name: "claw402_policy::tool::execute".to_string(),
                action,
                outcome: Some(outcome),
                duration_ms: None,
                attrs: None,
                message: message.to_string(),
            },
        );
    }

    export!(Claw402Policy);
}
