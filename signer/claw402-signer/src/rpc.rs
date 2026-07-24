//! Read-only trusted Solana RPC adapter.

use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};
use thiserror::Error;
use url::Url;

use crate::{BuildContext, TOKEN_2022_PROGRAM, TOKEN_PROGRAM};

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("RPC URL must use HTTPS")]
    InsecureUrl,
    #[error("invalid RPC URL")]
    InvalidUrl,
    #[error("Solana RPC request failed: {0}")]
    Transport(String),
    #[error("Solana RPC returned error {code}: {message}")]
    Remote { code: i64, message: String },
    #[error("Solana RPC response is missing result")]
    MissingResult,
    #[error("mint account does not exist")]
    MissingMint,
    #[error("mint account is not owned by SPL Token or Token-2022: {0}")]
    UnsupportedMintOwner(String),
    #[error("mint metadata response is malformed")]
    InvalidMintMetadata,
}

#[derive(Debug, Clone)]
pub struct TrustedRpcClient {
    endpoint: String,
}

impl TrustedRpcClient {
    pub fn new(endpoint: impl Into<String>) -> Result<Self, RpcError> {
        let endpoint = endpoint.into();
        let parsed = Url::parse(&endpoint).map_err(|_| RpcError::InvalidUrl)?;
        if parsed.scheme() != "https" {
            return Err(RpcError::InsecureUrl);
        }
        Ok(Self { endpoint })
    }

    /// Resolves all chain-controlled inputs required by the restricted builder.
    pub fn resolve_context(&self, mint: &str) -> Result<BuildContext, RpcError> {
        let latest: LatestBlockhashResult =
            self.call("getLatestBlockhash", json!([{ "commitment": "confirmed" }]))?;
        let account: AccountInfoResult = self.call(
            "getAccountInfo",
            json!([
                mint,
                {
                    "encoding": "jsonParsed",
                    "commitment": "confirmed"
                }
            ]),
        )?;
        let mint = account.value.ok_or(RpcError::MissingMint)?;
        if mint.owner != TOKEN_PROGRAM && mint.owner != TOKEN_2022_PROGRAM {
            return Err(RpcError::UnsupportedMintOwner(mint.owner));
        }
        let decimals = mint
            .data
            .parsed
            .info
            .decimals
            .ok_or(RpcError::InvalidMintMetadata)?;

        Ok(BuildContext {
            recent_blockhash: latest.value.blockhash,
            last_valid_block_height: latest.value.last_valid_block_height,
            token_program: mint.owner,
            mint_decimals: decimals,
        })
    }

    fn call<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<T, RpcError> {
        let mut response = ureq::post(&self.endpoint)
            .send_json(json!({
                "jsonrpc": "2.0",
                "id": "claw402",
                "method": method,
                "params": params
            }))
            .map_err(|error| RpcError::Transport(error.to_string()))?;
        let envelope: RpcEnvelope<T> = response
            .body_mut()
            .read_json()
            .map_err(|error| RpcError::Transport(error.to_string()))?;
        if let Some(error) = envelope.error {
            return Err(RpcError::Remote {
                code: error.code,
                message: error.message,
            });
        }
        envelope.result.ok_or(RpcError::MissingResult)
    }
}

#[derive(Debug, Deserialize)]
struct RpcEnvelope<T> {
    result: Option<T>,
    error: Option<RpcRemoteError>,
}

#[derive(Debug, Deserialize)]
struct RpcRemoteError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct LatestBlockhashResult {
    value: LatestBlockhashValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LatestBlockhashValue {
    blockhash: String,
    last_valid_block_height: u64,
}

#[derive(Debug, Deserialize)]
struct AccountInfoResult {
    value: Option<ParsedMintAccount>,
}

#[derive(Debug, Deserialize)]
struct ParsedMintAccount {
    owner: String,
    data: ParsedMintData,
}

#[derive(Debug, Deserialize)]
struct ParsedMintData {
    parsed: ParsedMint,
}

#[derive(Debug, Deserialize)]
struct ParsedMint {
    info: ParsedMintInfo,
}

#[derive(Debug, Deserialize)]
struct ParsedMintInfo {
    decimals: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_https_rpc_endpoints() {
        assert!(matches!(
            TrustedRpcClient::new("http://api.devnet.solana.com"),
            Err(RpcError::InsecureUrl)
        ));
    }

    #[test]
    fn parses_supported_mint_metadata_shape() {
        let response = json!({
            "value": {
                "owner": TOKEN_PROGRAM,
                "data": {
                    "parsed": {
                        "info": {
                            "decimals": 6
                        }
                    }
                }
            }
        });
        let parsed: AccountInfoResult = serde_json::from_value(response).unwrap();
        let mint = parsed.value.unwrap();
        assert_eq!(mint.owner, TOKEN_PROGRAM);
        assert_eq!(mint.data.parsed.info.decimals, Some(6));
    }
}
