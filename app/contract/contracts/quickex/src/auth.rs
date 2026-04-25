use crate::errors::QuickexError;
use crate::storage;
use crate::types::SignaturePayload;
use soroban_sdk::{Address, Env, IntoVal};

/// Verify a signature against a payload, ensuring replay protection and domain separation.
///
/// # Arguments
/// * `env` - The contract environment.
/// * `signer` - The address that signed the payload.
/// * `signature` - The signature to verify (e.g., Ed25519 signature).
/// * `payload` - The payload containing nonce, expiry, and params.
pub fn verify_signature(
    env: &Env,
    signer: &Address,
    signature: &soroban_sdk::BytesN<64>,
    payload: &SignaturePayload,
) -> Result<(), QuickexError> {
    // 1. Domain Separation: Check Network ID and Contract ID
    if payload.network_id != env.ledger().network_id() {
        return Err(QuickexError::Unauthorized);
    }
    if payload.contract_id != env.current_contract_id() {
        return Err(QuickexError::Unauthorized);
    }

    // 2. Replay Protection: Check and Increment Nonce
    let stored_nonce = storage::get_nonce(env, signer);
    if payload.nonce != stored_nonce {
        return Err(QuickexError::NonceMismatch);
    }
    storage::read_and_increment_nonce(env, signer.clone());

    // 3. Expiry Check: Check Ledger Sequence
    if env.ledger().sequence() > payload.expiry {
        return Err(QuickexError::SignatureExpired);
    }

    // 4. Cryptographic Verification: Verify Signature
    // We hash the payload using the env's crypto functions or by converting it to val.
    // verify_sig takes (public_key_address, message_hash, signature)
    signer.verify_sig(payload, signature);

    Ok(())
}
