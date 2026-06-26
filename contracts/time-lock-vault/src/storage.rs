use soroban_sdk::{Address, Env, Vec};

use crate::types::{VaultEntry, VaultKey};

// ----------------------------------------------------------------
//  Persistent storage TTL constants
// ----------------------------------------------------------------

pub const BUMP_THRESHOLD: u32 = 518_400;
pub const BUMP_TARGET: u32 = 33_000_000;

// ----------------------------------------------------------------
//  Deposit helpers
// ----------------------------------------------------------------

pub fn set_deposit(env: &Env, depositor: &Address, entry: &VaultEntry) {
    let key = VaultKey::Deposit(depositor.clone());
    env.storage().persistent().set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_deposit(env: &Env, depositor: &Address) -> Option<VaultEntry> {
    let key = VaultKey::Deposit(depositor.clone());
    let entry: Option<VaultEntry> = env.storage().persistent().get(&key);
    if entry.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    entry
}

pub fn get_deposit_readonly(env: &Env, depositor: &Address) -> Option<VaultEntry> {
    let key = VaultKey::Deposit(depositor.clone());
    env.storage().persistent().get(&key)
}

pub fn remove_deposit(env: &Env, depositor: &Address) {
    let key = VaultKey::Deposit(depositor.clone());
    env.storage().persistent().remove(&key);
}

// ----------------------------------------------------------------
//  Admin helpers
// ----------------------------------------------------------------

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&VaultKey::Admin, admin);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::Admin, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&VaultKey::Admin)
}

pub fn set_pending_admin(env: &Env, pending: &Address) {
    env.storage()
        .persistent()
        .set(&VaultKey::PendingAdmin, pending);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::PendingAdmin, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_pending_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&VaultKey::PendingAdmin)
}

pub fn remove_pending_admin(env: &Env) {
    env.storage().persistent().remove(&VaultKey::PendingAdmin);
}

// ----------------------------------------------------------------
//  Initialized flag
// ----------------------------------------------------------------

pub fn set_initialized(env: &Env) {
    env.storage()
        .persistent()
        .set(&VaultKey::Initialized, &true);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::Initialized, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&VaultKey::Initialized)
        .unwrap_or(false)
}

// ----------------------------------------------------------------
//  Runtime limits helpers
// ----------------------------------------------------------------

pub fn set_max_deposit(env: &Env, v: i128) {
    env.storage().persistent().set(&VaultKey::MaxDeposit, &v);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::MaxDeposit, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_max_deposit(env: &Env) -> Option<i128> {
    env.storage().persistent().get(&VaultKey::MaxDeposit)
}

pub fn set_max_lock_secs(env: &Env, v: u64) {
    env.storage().persistent().set(&VaultKey::MaxLockSecs, &v);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::MaxLockSecs, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_max_lock_secs(env: &Env) -> Option<u64> {
    env.storage().persistent().get(&VaultKey::MaxLockSecs)
}

// ----------------------------------------------------------------
//  Fee recipient helpers
// ----------------------------------------------------------------

pub fn set_fee_recipient(env: &Env, recipient: &Address) {
    env.storage()
        .persistent()
        .set(&VaultKey::FeeRecipient, recipient);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::FeeRecipient, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_fee_recipient(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&VaultKey::FeeRecipient)
}

// ----------------------------------------------------------------
//  Depositor index helpers  (O(1) add / O(1) remove via swap-remove)
// ----------------------------------------------------------------

fn get_depositor_count_raw(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&VaultKey::DepositorCount)
        .unwrap_or(0)
}

fn set_depositor_count(env: &Env, count: u32) {
    env.storage()
        .persistent()
        .set(&VaultKey::DepositorCount, &count);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::DepositorCount, BUMP_THRESHOLD, BUMP_TARGET);
}

fn get_depositor_at(env: &Env, slot: u32) -> Address {
    env.storage()
        .persistent()
        .get(&VaultKey::DepositorAt(slot))
        .unwrap()
}

fn set_depositor_at(env: &Env, slot: u32, addr: &Address) {
    env.storage()
        .persistent()
        .set(&VaultKey::DepositorAt(slot), addr);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::DepositorAt(slot), BUMP_THRESHOLD, BUMP_TARGET);
}

fn remove_depositor_at(env: &Env, slot: u32) {
    env.storage()
        .persistent()
        .remove(&VaultKey::DepositorAt(slot));
}

fn get_depositor_slot(env: &Env, addr: &Address) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&VaultKey::DepositorIndex(addr.clone()))
}

fn set_depositor_slot(env: &Env, addr: &Address, slot: u32) {
    env.storage()
        .persistent()
        .set(&VaultKey::DepositorIndex(addr.clone()), &slot);
    env.storage().persistent().extend_ttl(
        &VaultKey::DepositorIndex(addr.clone()),
        BUMP_THRESHOLD,
        BUMP_TARGET,
    );
}

fn remove_depositor_slot(env: &Env, addr: &Address) {
    env.storage()
        .persistent()
        .remove(&VaultKey::DepositorIndex(addr.clone()));
}

pub fn add_depositor(env: &Env, depositor: &Address) {
    let count = get_depositor_count_raw(env);
    set_depositor_at(env, count, depositor);
    set_depositor_slot(env, depositor, count);
    set_depositor_count(env, count + 1);
}

/// O(1) swap-remove: moves the last element into the vacated slot.
pub fn remove_depositor(env: &Env, depositor: &Address) {
    let count = get_depositor_count_raw(env);
    if count == 0 {
        return;
    }
    let slot = match get_depositor_slot(env, depositor) {
        Some(s) => s,
        None => return,
    };
    let last = count - 1;
    if slot != last {
        // Move last element into the freed slot
        let last_addr = get_depositor_at(env, last);
        set_depositor_at(env, slot, &last_addr);
        set_depositor_slot(env, &last_addr, slot);
    }
    remove_depositor_at(env, last);
    remove_depositor_slot(env, depositor);
    set_depositor_count(env, last);
}

pub fn get_depositor_count(env: &Env) -> u32 {
    get_depositor_count_raw(env)
}

pub fn get_depositors_page(env: &Env, offset: u32, limit: u32) -> Vec<Address> {
    let count = get_depositor_count_raw(env);
    let mut page: Vec<Address> = Vec::new(env);
    let end = (offset + limit).min(count);
    for i in offset..end {
        page.push_back(get_depositor_at(env, i));
    }
    page
}
