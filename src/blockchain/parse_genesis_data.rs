use std::collections::{HashMap, BTreeMap};
use serde_json;
use cardano::{config, fee, block, coin, redeem};
use base64;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct RawGenesisData {
    avvmDistr: HashMap<String, String>,
    nonAvvmBalances: HashMap<String, String>,
    protocolConsts: ProtocolConsts,
    blockVersionData: BlockVersionData,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct ProtocolConsts {
    k: usize,
    protocolMagic: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct BlockVersionData {
    txFeePolicy: TxFeePolicy,
}

#[derive(Deserialize, Debug)]
struct TxFeePolicy {
    summand: String,
    multiplier: String,
}

pub fn parse_genesis_data(json: &str) -> config::GenesisData { // FIXME: use Result

    let data: RawGenesisData = serde_json::from_str(&json).unwrap();

    let parse_fee_constant = |s: &str| {
        let n = s.parse::<u64>().unwrap();
        assert!(n % 1000000 == 0);
        fee::Milli(n / 1000000)
    };

    let mut avvm_distr = BTreeMap::new();
    for (avvm, balance) in &data.avvmDistr {
        avvm_distr.insert(
            redeem::PublicKey::from_slice(
                &base64::decode_config(avvm, base64::URL_SAFE).unwrap()).unwrap(),
            coin::Coin::new(balance.parse::<u64>().unwrap()).unwrap());
    }

    config::GenesisData {
        genesis_prev: block::HeaderHash::new(canonicalize_json(json).as_bytes()),
        epoch_stability_depth: data.protocolConsts.k,
        protocol_magic: config::ProtocolMagic::from(data.protocolConsts.protocolMagic),
        fee_policy: fee::LinearFee::new(
            parse_fee_constant(&data.blockVersionData.txFeePolicy.summand),
            parse_fee_constant(&data.blockVersionData.txFeePolicy.multiplier)),
        avvm_distr,
        non_avvm_balances: BTreeMap::new(), // FIXME
    }
}

pub fn canonicalize_json(json: &str) -> String
{
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    data.to_string()
}
