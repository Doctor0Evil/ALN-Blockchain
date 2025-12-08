use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OriginLockEvent {
    pub origin_chain_id: String,
    pub tx_hash: String,
    pub nonce: u64,
    pub denom: String,
    pub origin_address: String,
    pub amount: String,
    pub height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnergyVector {
    pub auet: Uint128,
    pub csp: Uint128,
    pub erp: Uint128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SanitizationDecision {
    Approved,
    Downgraded,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SanitizationResult {
    pub decision: SanitizationDecision,
    pub energy: EnergyVector,
    pub report_hash: Option<String>,
}

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("lightclient-error: {0}")]
    LightClient(String),
    #[error("ubs-error: {0}")]
    UBS(String),
    #[error("proof-error: {0}")]
    Proof(String),
    #[error("invariant-violation: {0}")]
    Invariant(String),
}

pub trait LightClient {
    /// Verify (proof + root) and return true if valid. Implementation should be deterministic.
    fn verify_root(&self, origin_chain_id: &str, root: &str, proof: &[u8]) -> Result<bool, BridgeError>;
}

pub trait UBS {
    /// Deterministically sanitize a token and compute an EnergyVector, writing a report to disk.
    fn sanitize(&self, origin_chain_id: &str, token_addr: &str, contract_wasm: &[u8]) -> Result<SanitizationResult, BridgeError>;
}

pub trait AlnBridge {
    /// Handles a bridge claim; requires origin event & UBS-sanitized mapping to ALN energy vector.
    fn claim(&self, origin_event: OriginLockEvent, merkle_proof: Vec<u8>, metadata_hash: Option<String>) -> Result<EnergyVector, BridgeError>;
}
