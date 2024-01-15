use alloy_rpc_types::Transaction as AlloyTx;
use ethers_core::types::{
    transaction::eip2930::AccessList, Address, Bytes, Transaction as EthersTx, H256, U256, U64,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
/// Eden-specific pending transaction type
pub struct EdenPendingTx {
    pub r#type: U64,
    pub hash: H256,
    #[serde(default = "ethers_core::types::Address::zero")]
    pub from: Address,
    pub nonce: U256,
    pub gas_limit: U256,
    pub to: Option<Address>,
    pub data: Bytes,
    pub v: U64,
    pub r: U256,
    pub s: U256,
    pub value: U256,
    pub chain_id: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<AccessList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,
}

impl EdenPendingTx {
    /// cast `EdenPendingTx` into ethers-rs transaction
    pub fn into_ethers_tx(self) -> EthersTx {
        EthersTx {
            hash: self.hash,
            nonce: self.nonce,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: self.from,
            to: self.to,
            value: self.value,
            gas_price: self.gas_price,
            gas: self.gas_limit,
            input: self.data,
            v: self.v,
            r: self.r,
            s: self.s,
            transaction_type: Some(self.r#type),
            access_list: self.access_list,
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas,
            chain_id: self.chain_id,
            ..Default::default()
        }
    }

    /// cast `EdenPendingTx` into alloy transaction
    pub fn into_alloy_tx(self) -> AlloyTx {
        unimplemented!("Not yet implemented");
    }
}

impl From<EdenPendingTx> for EthersTx {
    fn from(val: EdenPendingTx) -> Self {
        EthersTx {
            hash: val.hash,
            nonce: val.nonce,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            from: val.from,
            to: val.to,
            value: val.value,
            gas_price: val.gas_price,
            gas: val.gas_limit,
            input: val.data,
            v: val.v,
            r: val.r,
            s: val.s,
            transaction_type: Some(val.r#type),
            access_list: val.access_list,
            max_priority_fee_per_gas: val.max_priority_fee_per_gas,
            max_fee_per_gas: val.max_fee_per_gas,
            chain_id: val.chain_id,
            ..Default::default()
        }
    }
}
