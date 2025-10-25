#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InsufficientCollateral = 3,
    ExceedsCreditLimit = 4,
    InsufficientBalance = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPosition {
    pub collateral: i128,
    pub borrowed: i128,
    pub last_update: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    BenjiToken,
    UsdcToken,
    UserPosition(Address),
    LtvRatio, // 7000 = 70%
}

#[contract]
pub struct CreditLineContract;

#[contractimpl]
impl CreditLineContract {
    /// Initialize the contract
    pub fn initialize(
        env: Env,
        admin: Address,
        benji_token: Address,
        usdc_token: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::BenjiToken, &benji_token);
        env.storage()
            .instance()
            .set(&DataKey::UsdcToken, &usdc_token);
        env.storage().instance().set(&DataKey::LtvRatio, &7000_u32); // 70%

        Ok(())
    }

    /// Deposit BENJI tokens as collateral
    pub fn deposit_collateral(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        user.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Get BENJI token
        let benji_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::BenjiToken)
            .ok_or(Error::NotInitialized)?;

        // Transfer BENJI from user to contract
        let token_client = token::Client::new(&env, &benji_token);
        token_client.transfer(&user, &env.current_contract_address(), &amount);

        // Update user position
        let mut position: UserPosition = env
            .storage()
            .persistent()
            .get(&DataKey::UserPosition(user.clone()))
            .unwrap_or(UserPosition {
                collateral: 0,
                borrowed: 0,
                last_update: env.ledger().timestamp(),
            });

        position.collateral += amount;
        position.last_update = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::UserPosition(user), &position);

        Ok(())
    }

    /// Borrow USDC against BENJI collateral
    pub fn borrow(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        user.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Get user position
        let mut position: UserPosition = env
            .storage()
            .persistent()
            .get(&DataKey::UserPosition(user.clone()))
            .ok_or(Error::InsufficientCollateral)?;

        // Calculate credit limit (70% of collateral value)
        let ltv_ratio: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LtvRatio)
            .unwrap_or(7000);

        let credit_limit = (position.collateral * ltv_ratio as i128) / 10000;

        // Check if borrow amount is within limit
        if position.borrowed + amount > credit_limit {
            return Err(Error::ExceedsCreditLimit);
        }

        // Get USDC token
        let usdc_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .ok_or(Error::NotInitialized)?;

        // Transfer USDC to user
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&env.current_contract_address(), &user, &amount);

        // Update position
        position.borrowed += amount;
        position.last_update = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::UserPosition(user), &position);

        Ok(())
    }

    /// Repay borrowed USDC
    pub fn repay(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        user.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Get user position
        let mut position: UserPosition = env
            .storage()
            .persistent()
            .get(&DataKey::UserPosition(user.clone()))
            .ok_or(Error::NotInitialized)?;

        if position.borrowed < amount {
            panic!("Repay amount exceeds borrowed amount");
        }

        // Get USDC token
        let usdc_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::UsdcToken)
            .ok_or(Error::NotInitialized)?;

        // Transfer USDC from user to contract
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&user, &env.current_contract_address(), &amount);

        // Update position
        position.borrowed -= amount;
        position.last_update = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::UserPosition(user), &position);

        Ok(())
    }

    /// Withdraw collateral (only if enough collateral remains)
    pub fn withdraw_collateral(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        user.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Get user position
        let mut position: UserPosition = env
            .storage()
            .persistent()
            .get(&DataKey::UserPosition(user.clone()))
            .ok_or(Error::NotInitialized)?;

        if position.collateral < amount {
            return Err(Error::InsufficientBalance);
        }

        // Check if remaining collateral covers borrowed amount
        let new_collateral = position.collateral - amount;
        let ltv_ratio: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LtvRatio)
            .unwrap_or(7000);

        let credit_limit = (new_collateral * ltv_ratio as i128) / 10000;

        if position.borrowed > credit_limit {
            return Err(Error::InsufficientCollateral);
        }

        // Get BENJI token
        let benji_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::BenjiToken)
            .ok_or(Error::NotInitialized)?;

        // Transfer BENJI back to user
        let token_client = token::Client::new(&env, &benji_token);
        token_client.transfer(&env.current_contract_address(), &user, &amount);

        // Update position
        position.collateral -= amount;
        position.last_update = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::UserPosition(user), &position);

        Ok(())
    }

    /// Get user's position
    pub fn get_position(env: Env, user: Address) -> UserPosition {
        env.storage()
            .persistent()
            .get(&DataKey::UserPosition(user))
            .unwrap_or(UserPosition {
                collateral: 0,
                borrowed: 0,
                last_update: env.ledger().timestamp(),
            })
    }

    /// Calculate available credit for a user
    pub fn get_available_credit(env: Env, user: Address) -> i128 {
        let position = Self::get_position(env.clone(), user);

        let ltv_ratio: u32 = env
            .storage()
            .instance()
            .get(&DataKey::LtvRatio)
            .unwrap_or(7000);

        let credit_limit = (position.collateral * ltv_ratio as i128) / 10000;
        let available = credit_limit - position.borrowed;

        if available < 0 {
            0
        } else {
            available
        }
    }
}
