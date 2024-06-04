mod beacon_block;
mod blinded_block_body;
mod blobs_bundle;
mod execution_payload;
mod get_header;
mod kzg;
mod spec;
mod utils;

pub use beacon_block::{SignedBlindedBeaconBlock, SubmitBlindedBlockResponse};
pub use execution_payload::EMPTY_TX_ROOT_HASH;
pub use get_header::{GetHeaderParams, GetHeaderReponse, SignedExecutionPayloadHeader};

// ethereum consensus -> shouldnt be used in prod
// alloy doesnt have all consensus types
// lighthouse most complete but very heavy, doesnt use alloy
// ethereum types has u256 and address which works with TreeHash
