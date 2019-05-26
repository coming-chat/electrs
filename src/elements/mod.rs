use bitcoin::blockdata::script::Instruction::PushBytes;
use bitcoin::consensus::encode::serialize;
use bitcoin::Script;
use bitcoin_hashes::hex::ToHex;
use elements::confidential::Value;
use elements::{AssetIssuance, Proof, TxIn};
use hex;

use crate::chain::Network;
use crate::util::{get_script_asm, script_to_address};

pub mod asset;
mod assetid;
mod registry;

use asset::get_issuance_assetid;
pub use asset::{lookup_asset, AssetEntry};
pub use assetid::AssetId;
pub use registry::AssetRegistry;

#[derive(Serialize, Deserialize)]
pub struct BlockProofValue {
    challenge: Script,
    challenge_asm: String,
    solution: Script,
    solution_asm: String,
}

impl From<&Proof> for BlockProofValue {
    fn from(proof: &Proof) -> Self {
        BlockProofValue {
            challenge_asm: get_script_asm(&proof.challenge),
            challenge: proof.challenge.clone(),
            solution_asm: get_script_asm(&proof.solution),
            solution: proof.solution.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PegOutRequest {
    pub genesis_hash: String,
    pub scriptpubkey: Script,
    pub scriptpubkey_asm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scriptpubkey_address: Option<String>,
}

impl PegOutRequest {
    pub fn parse(
        script: &Script,
        parent_network: &Network,
        parent_genesis_hash: &str,
    ) -> Option<PegOutRequest> {
        if !script.is_op_return() {
            return None;
        }

        let nulldata: Vec<_> = script.iter(true).skip(1).collect();
        if nulldata.len() < 2 {
            return None;
        }

        let genesis_hash = if let PushBytes(data) = nulldata[0] {
            let mut data = data.to_vec();
            data.reverse();
            hex::encode(data)
        } else {
            return None;
        };

        let scriptpubkey = if let PushBytes(data) = nulldata[1] {
            Script::from(data.to_vec())
        } else {
            return None;
        };

        if genesis_hash != parent_genesis_hash {
            return None;
        }

        let scriptpubkey_asm = get_script_asm(&scriptpubkey);
        let scriptpubkey_address = script_to_address(&scriptpubkey, parent_network);

        Some(PegOutRequest {
            genesis_hash,
            scriptpubkey,
            scriptpubkey_asm,
            scriptpubkey_address,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IssuanceValue {
    pub asset_id: String,
    pub is_reissuance: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_blinding_nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_entropy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assetamount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assetamountcommitment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenamount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenamountcommitment: Option<String>,
}

impl From<&TxIn> for IssuanceValue {
    fn from(txin: &TxIn) -> Self {
        let asset_id = get_issuance_assetid(txin).expect("invalid issuance");
        IssuanceValue::new(asset_id.to_hex(), &txin.asset_issuance)
    }
}

impl IssuanceValue {
    fn new(asset_id: String, issuance: &AssetIssuance) -> Self {
        let zero = [0u8; 32];
        let is_reissuance = issuance.asset_blinding_nonce != zero;

        IssuanceValue {
            asset_id,
            is_reissuance,
            asset_blinding_nonce: if is_reissuance {
                Some(hex::encode(issuance.asset_blinding_nonce))
            } else {
                None
            },
            asset_entropy: if issuance.asset_entropy != zero {
                // reverse to match the format used by elements-cpp
                let mut entropy = issuance.asset_entropy;
                entropy.reverse();
                Some(hex::encode(entropy))
            } else {
                None
            },
            assetamount: match issuance.amount {
                Value::Explicit(value) => Some(value),
                _ => None,
            },
            assetamountcommitment: match issuance.amount {
                Value::Confidential(..) => Some(hex::encode(serialize(&issuance.amount))),
                _ => None,
            },
            tokenamount: match issuance.inflation_keys {
                Value::Explicit(value) => Some(value),
                _ => None,
            },
            tokenamountcommitment: match issuance.inflation_keys {
                Value::Confidential(..) => Some(hex::encode(serialize(&issuance.inflation_keys))),
                _ => None,
            },
        }
    }
}
