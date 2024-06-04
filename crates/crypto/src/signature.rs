use alloy_rpc_types_beacon::{constants::BLS_DST_SIG, BlsPublicKey, BlsSignature};
use blst::{
    min_pk::{PublicKey, SecretKey, Signature},
    BLST_ERROR,
};
use cb_common::types::Chain;
use rand::RngCore;
use ssz_derive::{Decode, Encode};
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

use crate::{
    types::ObjectTreeHash,
    utils::{alloy_pubkey_to_blst, alloy_sig_to_blst},
};

pub fn random_secret() -> SecretKey {
    let mut rng = rand::thread_rng();
    let mut ikm = [0u8; 32];
    rng.fill_bytes(&mut ikm);
    SecretKey::key_gen(&ikm, &[]).unwrap()
}

pub fn verify_signature(
    pubkey: &BlsPublicKey,
    msg: &[u8],
    signature: &BlsSignature,
) -> Result<(), blst::BLST_ERROR> {
    let pubkey: PublicKey = alloy_pubkey_to_blst(pubkey)?;
    let signature: Signature = alloy_sig_to_blst(signature)?;

    let res = signature.verify(true, msg, BLS_DST_SIG, &[], &pubkey, true);
    if res == BLST_ERROR::BLST_SUCCESS {
        Ok(())
    } else {
        Err(res)
    }
}

pub fn sign_message(secret_key: &SecretKey, msg: &[u8]) -> BlsSignature {
    let signature = secret_key.sign(msg, BLS_DST_SIG, &[]).to_bytes();
    BlsSignature::from_slice(&signature)
}

#[derive(Default, Debug, Encode, Decode, TreeHash)]
struct SigningData {
    object_root: [u8; 32],
    signing_domain: [u8; 32],
}

pub fn compute_signing_root(msg: &impl ObjectTreeHash, signing_domain: [u8; 32]) -> [u8; 32] {
    let object_root = msg.tree_hash().0;
    let signing_data = SigningData { object_root, signing_domain };
    signing_data.tree_hash_root().0
}

const APPLICATION_BUILDER_DOMAIN: [u8; 4] = [0, 0, 0, 1];
const MAINNET_FORK_VERSION: [u8; 4] = [0u8; 4];
const HOLESKY_FORK_VERSION: [u8; 4] = [1, 1, 112, 0];
const GENESIS_VALIDATORS_ROOT: [u8; 32] = [0; 32];

const MAINNET_BUILDER_DOMAIN: [u8; 32] = [
    0, 0, 0, 1, 245, 165, 253, 66, 209, 106, 32, 48, 39, 152, 239, 110, 211, 9, 151, 155, 67, 0,
    61, 35, 32, 217, 240, 232, 234, 152, 49, 169,
];

const HOLESKY_BUILDER_DOMAIN: [u8; 32] = [
    0, 0, 0, 1, 91, 131, 162, 55, 89, 197, 96, 178, 208, 198, 69, 118, 225, 220, 252, 52, 234, 148,
    196, 152, 143, 62, 13, 159, 119, 240, 83, 135,
];

#[derive(Debug, Encode, Decode, TreeHash)]
struct ForkData {
    fork_version: [u8; 4],
    genesis_validators_root: [u8; 32],
}

#[allow(dead_code)]
fn compute_builder_domain(chain: Chain) -> [u8; 32] {
    let mut domain = [0u8; 32];
    domain[..4].copy_from_slice(&APPLICATION_BUILDER_DOMAIN);

    let fork_version = match chain {
        Chain::Mainnet => MAINNET_FORK_VERSION,
        Chain::Holesky => HOLESKY_FORK_VERSION,
    };
    let fd = ForkData { fork_version, genesis_validators_root: GENESIS_VALIDATORS_ROOT };
    let fork_data_root = fd.tree_hash_root();

    domain[4..].copy_from_slice(&fork_data_root[..28]);

    domain
}

pub fn verify_signed_builder_message<T: TreeHash>(
    chain: Chain,
    pubkey: &BlsPublicKey,
    msg: &T,
    signature: &BlsSignature,
) -> Result<(), BLST_ERROR> {
    let domain = match chain {
        Chain::Mainnet => MAINNET_BUILDER_DOMAIN,
        Chain::Holesky => HOLESKY_BUILDER_DOMAIN,
    };

    let signing_root = compute_signing_root(msg, domain);

    verify_signature(pubkey, &signing_root, signature)
}

pub fn sign_builder_message(
    chain: Chain,
    secret_key: &SecretKey,
    msg: &impl ObjectTreeHash,
) -> BlsSignature {
    let domain = match chain {
        Chain::Mainnet => MAINNET_BUILDER_DOMAIN,
        Chain::Holesky => HOLESKY_BUILDER_DOMAIN,
    };

    let signing_root = compute_signing_root(msg, domain);
    sign_message(secret_key, &signing_root)
}

#[cfg(test)]
mod tests {
    use cb_common::types::Chain;

    use super::compute_builder_domain;
    use crate::signature::{HOLESKY_BUILDER_DOMAIN, MAINNET_BUILDER_DOMAIN};

    #[test]
    fn test_builder_domains() {
        assert_eq!(compute_builder_domain(Chain::Mainnet), MAINNET_BUILDER_DOMAIN);
        assert_eq!(compute_builder_domain(Chain::Holesky), HOLESKY_BUILDER_DOMAIN);
    }
}
