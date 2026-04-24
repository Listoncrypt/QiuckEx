//! # QuickEx Storage Schema
//!
//! This module defines the persistent storage layout for the QuickEx contract.
//! All long-term data is stored via the [`DataKey`] enum, which centralises key
//! construction and ensures type-safe storage access.

use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Vec};
use crate::types::{EscrowEntry, FeeConfig, Role, StealthEscrowEntry};

// -----------------------------------------------------------------------------
// Key constants
// -----------------------------------------------------------------------------

pub const PRIVACY_ENABLED_KEY: &str = "privacy_enabled";
pub const LEDGER_THRESHOLD: u32 = 17280; // ~1 day
pub const SIX_MONTHS_IN_LEDGERS: u32 = 3110400; // ~185 days

/// Bitmask flags for granular operation pausing.
#[contracttype]
#[repr(u64)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PauseFlag {
    Deposit = 1,
    Withdrawal = 2,
    Refund = 4,
    DepositWithCommitment = 8,
    SetPrivacy = 16,
    CreateAmountCommitment = 32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Escrow(Bytes),
    EscrowCounter,
    Admin,
    Paused,
Pause,
    /// Numeric privacy level per account.
    PrivacyLevel(Address),
    PrivacyHistory(Address),
    StealthEscrow(BytesN<32>),
    PauseFlags,
    FeeConfig,
    PlatformWallet,
    /// Nonce for signature replay protection.
    Nonce(Address),
    /// Maps a deterministic 32-byte `escrow_id` (see [`crate::escrow_id`])
    /// to the commitment key of the escrow it identifies. Enables
    /// idempotent deduplication of identical creation requests.
    EscrowIdMap(BytesN<32>),
    /// Roles assigned to an address.
    UserRole(Address),
}

// -----------------------------------------------------------------------------
// Escrow helpers
// -----------------------------------------------------------------------------

pub fn put_escrow(env: &Env, commitment: &Bytes, entry: &EscrowEntry) {
    let key = DataKey::Escrow(commitment.clone());
    env.storage().persistent().set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, SIX_MONTHS_IN_LEDGERS);
}

pub fn remove_escrow(env: &Env, commitment: &Bytes) {
    let key = DataKey::Escrow(commitment.clone());
    env.storage().persistent().remove(&key);
}

pub fn get_escrow(env: &Env, commitment: &Bytes) -> Option<EscrowEntry> {
    let key = DataKey::Escrow(commitment.clone());
    env.storage().persistent().get(&key)
}

pub fn has_escrow(env: &Env, commitment: &Bytes) -> bool {
    let key = DataKey::Escrow(commitment.clone());
    env.storage().persistent().has(&key)
}

pub fn get_escrow_counter(env: &Env) -> u64 {
    let key = DataKey::EscrowCounter;
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn increment_escrow_counter(env: &Env) -> u64 {
    let key = DataKey::EscrowCounter;
    let mut count: u64 = env.storage().persistent().get(&key).unwrap_or(0);
    count += 1;
    env.storage().persistent().set(&key, &count);
    count
}

// -----------------------------------------------------------------------------
// Admin & Pause helpers
// -----------------------------------------------------------------------------

pub fn set_admin(env: &Env, admin: &Address) {
    let key = DataKey::Admin;
    env.storage().persistent().set(&key, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    let key = DataKey::Admin;
    env.storage().persistent().get(&key)
}

pub fn set_paused(env: &Env, paused: bool) {
    let key = DataKey::Paused;
    env.storage().persistent().set(&key, &paused);
}

/// Set pause flags (granular pause control – caller already verified by admin module).
pub fn set_pause_flags(env: &Env, _caller: &Address, flags_to_enable: u64, flags_to_disable: u64) {
    let key = DataKey::PauseFlags;
    let current: u64 = env.storage().persistent().get(&key).unwrap_or(0);
    let updated = (current | flags_to_enable) & !flags_to_disable;
    env.storage().persistent().set(&key, &updated);
}

/// Get the global paused state.
pub fn is_paused(env: &Env) -> bool {
    let key = DataKey::Paused;
    env.storage().persistent().get(&key).unwrap_or(false)
}
pub fn set_pause_flags(env: &Env, _caller: &Address, flags_to_enable: u64, flags_to_disable: u64) {
    let key = DataKey::PauseFlags;
    let current: u64 = env.storage().persistent().get(&key).unwrap_or(0);
    let updated = (current | flags_to_enable) & !flags_to_disable;
    env.storage().persistent().set(&key, &updated);
}

pub fn is_feature_paused(env: &Env, flag: PauseFlag) -> bool {
    let key = DataKey::PauseFlags;
    let flags: u64 = env.storage().persistent().get(&key).unwrap_or(0);
    flags & (flag as u64) != 0
}

// -----------------------------------------------------------------------------
// Privacy & Stealth helpers
// -----------------------------------------------------------------------------

pub fn set_privacy_level(env: &Env, account: &Address, level: u32) {
    let key = DataKey::PrivacyLevel(account.clone());
    env.storage().persistent().set(&key, &level);
}

pub fn get_privacy_level(env: &Env, account: &Address) -> Option<u32> {
    let key = DataKey::PrivacyLevel(account.clone());
    env.storage().persistent().get(&key)
}

pub fn add_privacy_history(env: &Env, account: &Address, level: u32) {
    let key = DataKey::PrivacyHistory(account.clone());
    let mut history: Vec<u32> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    history.push_front(level);
    env.storage().persistent().set(&key, &history);
}

pub fn get_privacy_history(env: &Env, account: &Address) -> Vec<u32> {
    let key = DataKey::PrivacyHistory(account.clone());
    env.storage().persistent().get(&key).unwrap_or(Vec::new(env))
}

pub fn get_stealth_escrow(env: &Env, stealth_address: &BytesN<32>) -> Option<StealthEscrowEntry> {
    let key = DataKey::StealthEscrow(stealth_address.clone());
    env.storage().persistent().get(&key)
}

pub fn put_stealth_escrow(env: &Env, stealth_address: &BytesN<32>, entry: &StealthEscrowEntry) {
    let key = DataKey::StealthEscrow(stealth_address.clone());
    env.storage().persistent().set(&key, entry);
    env.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD, SIX_MONTHS_IN_LEDGERS);
}

// -----------------------------------------------------------------------------
// Fee & Wallet helpers
// -----------------------------------------------------------------------------

pub fn get_fee_config(env: &Env) -> FeeConfig {
    env.storage().persistent().get(&DataKey::FeeConfig).unwrap_or(FeeConfig { fee_bps: 0 })
}

pub fn set_fee_config(env: &Env, config: &FeeConfig) {
    env.storage().persistent().set(&DataKey::FeeConfig, config);
}

pub fn get_platform_wallet(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::PlatformWallet)
}

pub fn set_platform_wallet(env: &Env, wallet: &Address) {
    env.storage().persistent().set(&DataKey::PlatformWallet, wallet);
}

// -----------------------------------------------------------------------------
// Nonce & Escrow Mapping helpers
// -----------------------------------------------------------------------------

pub fn get_nonce(env: &Env, signer: &Address) -> u64 {
    let key = DataKey::Nonce(signer.clone());
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn read_and_increment_nonce(env: &Env, signer: Address) -> u64 {
    let key = DataKey::Nonce(signer.clone());
    let nonce: u64 = env.storage().persistent().get(&key).unwrap_or(0);
    env.storage().persistent().set(&key, &(nonce + 1));
    env.storage().persistent().extend_ttl(&key, LEDGER_THRESHOLD, SIX_MONTHS_IN_LEDGERS);
    nonce
}

// -----------------------------------------------------------------------------
// Role helpers
// -----------------------------------------------------------------------------

pub fn get_roles(env: &Env, address: &Address) -> Vec<Role> {
    let key = DataKey::UserRole(address.clone());
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(Vec::new(env))
}

pub fn set_roles(env: &Env, address: &Address, roles: &Vec<Role>) {
    let key = DataKey::UserRole(address.clone());
    env.storage().persistent().set(&key, roles);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, SIX_MONTHS_IN_LEDGERS);
}

// -----------------------------------------------------------------------------
// Escrow-id map helpers (Issue #304)
// -----------------------------------------------------------------------------

/// Look up the 32-byte commitment associated with a deterministic `escrow_id`.
pub fn get_escrow_id_mapping(env: &Env, escrow_id: &BytesN<32>) -> Option<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::EscrowIdMap(escrow_id.clone()))
}

/// Record the mapping `escrow_id → commitment` so future identical creates
/// can be recognized and deduplicated.
pub fn put_escrow_id_mapping(env: &Env, escrow_id: &BytesN<32>, commitment: &BytesN<32>) {
    let key = DataKey::EscrowIdMap(escrow_id.clone());
    env.storage().persistent().set(&key, commitment);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, SIX_MONTHS_IN_LEDGERS);
}
