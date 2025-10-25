#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token::TokenInterface, Address, Env, String,
};
use soroban_token_sdk::metadata::TokenMetadata;

#[contracttype]
pub enum DataKey {
    Admin,
    Metadata,
    Balance(Address),
    TotalSupply,
}

#[contract]
pub struct UsdcToken;

#[contractimpl]
impl UsdcToken {
    pub fn initialize(env: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        if decimal > 18 {
            panic!("Decimal must not be greater than 18");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(
            &DataKey::Metadata,
            &TokenMetadata {
                decimal,
                name,
                symbol,
            },
        );
        env.storage().instance().set(&DataKey::TotalSupply, &0_i128);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        admin.require_auth();

        if amount < 0 {
            panic!("Amount must be non-negative");
        }

        let balance = Self::balance(env.clone(), to.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &(balance + amount));

        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalSupply)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &(total + amount));
    }
}

#[contractimpl]
impl TokenInterface for UsdcToken {
    fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 {
        0
    }

    fn approve(
        _env: Env,
        _from: Address,
        _spender: Address,
        _amount: i128,
        _expiration_ledger: u32,
    ) {
        panic!("Not implemented");
    }

    fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(id))
            .unwrap_or(0)
    }

    fn transfer(env: Env, from: Address, to_muxed: soroban_sdk::MuxedAddress, amount: i128) {
        from.require_auth();

        if amount < 0 {
            panic!("Amount must be non-negative");
        }

        let to = to_muxed.address();

        let from_balance = Self::balance(env.clone(), from.clone());
        let to_balance = Self::balance(env.clone(), to.clone());

        if from_balance < amount {
            panic!("Insufficient balance");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_balance + amount));
    }

    fn transfer_from(_env: Env, _spender: Address, _from: Address, _to: Address, _amount: i128) {
        panic!("Not implemented");
    }

    fn burn(_env: Env, _from: Address, _amount: i128) {
        panic!("Not implemented");
    }

    fn burn_from(_env: Env, _spender: Address, _from: Address, _amount: i128) {
        panic!("Not implemented");
    }

    fn decimals(env: Env) -> u32 {
        let metadata: TokenMetadata = env
            .storage()
            .instance()
            .get(&DataKey::Metadata)
            .expect("Not initialized");
        metadata.decimal
    }

    fn name(env: Env) -> String {
        let metadata: TokenMetadata = env
            .storage()
            .instance()
            .get(&DataKey::Metadata)
            .expect("Not initialized");
        metadata.name
    }

    fn symbol(env: Env) -> String {
        let metadata: TokenMetadata = env
            .storage()
            .instance()
            .get(&DataKey::Metadata)
            .expect("Not initialized");
        metadata.symbol
    }
}
