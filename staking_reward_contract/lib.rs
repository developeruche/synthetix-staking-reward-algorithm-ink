#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]




#[openbrush::contract]
pub mod staking_reward_contract {


    // =====================================
    // MAKING IMPORTATIONS 
    // =====================================
    use openbrush::{
        contracts::{
            traits::psp22::PSP22Ref,
        },
    }; // this would be used for psp22 token interaction 
    use ink::{storage::Mapping};
    use ink::env::CallFlags;
    use ink::prelude::vec;



    // =================================
    // DEFINING CONTRACT STORAGE 
    // =================================

    #[ink(storage)]
    pub struct Contract {
        admin: AccountId,
        staked_token: AccountId,
        reward_token: AccountId,
        period_to_finish: Balance,
        reward_rate: Balance,
        reward_duration: Balance,
        last_updated_time: Balance,
        reward_per_token_stored: Balance,
        user_reward_per_token: Mapping<AccountId, Balance>,
        rewards: Mapping<AccountId, Balance>,
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
    }
    
    // ======================================
    // ADDING TOKEN EXTENSION TRAIT (default)
    // ======================================




    // =========================================
    // ERROR DECLARATION 
    // =========================================
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotAdmin,
        AmountShouldBeGreaterThanZero,
        InsufficientFunds,
        NotEnoughAllowance,
        TokenTransferFailed,
        Overflow,
        StakingStillInProgress
    }



    // ===================================
    // EVENTS DECLARATION 
    // ===================================
    #[ink(event)]
    pub struct Staked {
        #[ink(topic)]
        caller:AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct Withdraw {
        #[ink(topic)]
        caller:AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct RewardPaid {
        #[ink(topic)]
        caller:AccountId,
        reward: Balance,
    }

    #[ink(event)]
    pub struct RewardNotified {
        reward: Balance,
    }

    #[ink(event)]
    pub struct DurationUpdate {
        duration: Balance,
    }

    #[ink(impl)]
    impl Contract {
        // ================================================
        // MODIFIERS, AUTHORIZATION GATE AND SANITY CHECKS 
        // ================================================
        
        fn only_owner(&self) -> Result<(), Error> {
            if self.env().caller() == self.admin {
                Ok(())
            } else {
                Err(Error::NotAdmin)
            }
        }

        fn update_reward(&mut self, account: AccountId) {
            self.reward_per_token_stored = self.reward_per_token();
            self.last_updated_time = self.last_time_reward_applicable();

            if account == self.zero_address() {
                self.rewards.insert(account, &(self.earned(account)));
                self.user_reward_per_token.insert(account, &(self.reward_per_token_stored));
            }
        }

        fn zero_address(&self) -> AccountId {
            [0u8; 32].into()
        }

        fn transfer(
            &self,
            to: AccountId,
            token: AccountId,
            amount: Balance
        ) -> Result<(), Error> { 
            PSP22Ref::transfer(
                &token,
                to,
                amount,
                vec![])
                    .unwrap_or_else(|error| {
                    panic!(
                        "Failed to transfer PSP22 2 tokens to caller : {:?}",
                        error
                    )
            });

            Ok(())
        }


        fn transfer_from(
            &self,
            from: AccountId,
            to: AccountId,
            token: AccountId,
            amount: Balance
        ) -> Result<(), Error> { 
            // checking the balance of the sender to see if the sender has enough balance to run this transfer 
            let user_current_balance = PSP22Ref::balance_of(
                &token,
                from
            );

            if user_current_balance < amount {
                return Err(Error::InsufficientFunds)
            }

            // checking if enough allowance has been made for this operation 
            let staking_contract_allowance = PSP22Ref::allowance(
                &token,
                from,
                to
            );

            if staking_contract_allowance < amount {
                return Err(Error::NotEnoughAllowance)
            }

            let staking_contract_initial_balance = PSP22Ref::balance_of(
                &token,
                to
            );


            // making the transfer call to the token contract 
            if PSP22Ref::transfer_from_builder(
                &token,
                from,
                to,
                amount,
                vec![])
                    .call_flags(CallFlags::default()
                    .set_allow_reentry(true))
                    .try_invoke()
                    .expect("Transfer failed")
                    .is_err(){
                        return Err(Error::TokenTransferFailed);
            }

            let staking_contract_balance_after_transfer = PSP22Ref::balance_of(
                &token,
                to
            );

            let mut actual_token_staked:Balance = 0;
        
            // calculating the actual amount that came in to the contract, some token might have taxes, just confirming transfer for economic safety
            match staking_contract_balance_after_transfer.checked_sub(staking_contract_initial_balance) {
                Some(result) => {
                    actual_token_staked = result;
                }
                None => {
                    return Err(Error::Overflow);
                }
            };

            Ok(())
        }
    }

    #[ink(impl)]
    impl Contract {
        // =====================================
        // LIB
        // =====================================
        
        fn min(&self, x: Balance,y: Balance) -> Balance {
            if x < y {
                x
            } else {
                y
            }
        }


    }

    


    #[ink(impl)]
    impl Contract {
        // ====================================
        // MAIN IMPLEMENTATION BLOCK 
        // ====================================
        #[ink(constructor)]
        pub fn new(
            reward_token: AccountId,
            staked_token: AccountId,
            reward_duration: u128
        ) -> Self {
			Self {
                admin: Self::env().caller(),
                staked_token,
                reward_token,
                period_to_finish: 0,
                reward_rate: 0,
                reward_duration,
                last_updated_time: 0,
                reward_per_token_stored: 0,
                user_reward_per_token: Mapping::default(),
                rewards: Mapping::default(),
                total_supply: 0,
                balances: Mapping::default(),
            }
        }


        // ===============================
        // VIEW FUNCTIONS (MESSAGES)
        // ===============================

        #[ink(message)]
        pub fn total_supply(
            &self
        ) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(
            &self,
            account: AccountId
        ) -> Balance {
            self.balances.get(account).unwrap_or(0)
        }

        #[ink(message)]
        pub fn last_time_reward_applicable(
            &self
        ) -> Balance {
            self.min(self.env().block_timestamp() as u128, self.period_to_finish)
        }

        #[ink(message)]
        pub fn reward_per_token(
            &self
        ) -> Balance {
            let rpts = if self.total_supply == 0 {
                self.reward_per_token_stored
            } else {
                self.reward_per_token_stored + (
                    (
                        (
                            self.last_time_reward_applicable() - self.last_updated_time
                        ) * self.reward_rate
                    ) * 1000000000000000000
                ) / self.total_supply
            };
            
            rpts
        }

        #[ink(message)]
        pub fn earned(
            &self,
            account: AccountId
        ) -> Balance {
            (
                (
                    self.balances.get(account).unwrap_or(0) * (
                        self.reward_per_token() - self.user_reward_per_token.get(account).unwrap_or(0)
                    )
                ) / 1000000000000000000
            ) + self.rewards.get(account).unwrap_or(0)
        }

        #[ink(message)]
        pub fn get_reward_for_duration(
            &self
        ) -> Balance {
            self.reward_rate * self.reward_duration
        }

        #[ink(message)]
        pub fn return_address_zero(
            &self
        ) -> AccountId {
            self.zero_address()
        }


        // ===============================
        // WRITE FUNCTIONS
        // ===============================

        #[ink(message)]
        pub fn stake(
            &mut self,
            amount: Balance
        ) -> Result<(), Error> {
            let account = self.env().caller();
            self.update_reward(account);

            if amount <= 0 {
                return Err(Error::AmountShouldBeGreaterThanZero);
            }
            self.total_supply += amount;
            self.balances.insert(account, &(self.balance_of(account) + amount));

            self.transfer_from(account, self.env().account_id(), self.staked_token, amount)?;

            self.env().emit_event(
                Staked {
                    caller: account,
                    amount
                }
            );

            Ok(())
        }

        #[ink(message)]
        pub fn withdraw(
            &mut self,
            amount: Balance
        ) -> Result<(), Error> {
            let account = self.env().caller();
            self.update_reward(account);

            if amount <= 0 {
                return Err(Error::AmountShouldBeGreaterThanZero);
            }

            self.total_supply -= amount;
            self.balances.insert(account, &(self.balance_of(account) - amount));

            self.transfer(self.staked_token, account, amount)?;

            self.env().emit_event(
                Withdraw {
                    caller: account,
                    amount
                }
            );

            Ok(())
        }

        #[ink(message)]
        pub fn get_reward(
            &mut self
        ) -> Result<(), Error> {
            let account = self.env().caller();
            let reward = self.rewards.get(account).unwrap_or(0);
            self.update_reward(account);

            if reward > 0 {
                self.rewards.insert(account, &(0));
                self.transfer(account, self.reward_token, reward)?;

                self.env().emit_event(
                    RewardPaid {
                        caller: account,
                        reward
                    }
                );
            }

            Ok(())
        }

        #[ink(message)]
        pub fn exit(
            &mut self
        ) -> Result<(), Error> {
            let account = self.env().caller();
            let balance = self.balances.get(account).unwrap_or(0);
            self.withdraw(balance)?;
            self.get_reward()?;

            Ok(())
        }

        //===============================
        // GATED FUNCTIONS
        // ==============================

        #[ink(message)]
        pub fn notify_reward_amount(
            &mut self,
            reward: Balance
        ) -> Result<(), Error> {
            self.only_owner()?;
            self.update_reward(self.zero_address());
            let account = self.env().caller();

            // transferring the reward token from the admin to the staking contract
            self.transfer_from(account, self.env().account_id(), self.reward_token, reward)?;

            if self.env().block_timestamp() as u128 >= self.period_to_finish {
                // this means the staking period has not started 
                self.reward_rate = reward / self.reward_duration;
            } else {
                let remaining_staking_time = self.period_to_finish - self.env().block_timestamp() as u128;
                let left_over_reward = remaining_staking_time * self.reward_rate;

                self.reward_rate = (reward + left_over_reward) / self.reward_duration;
            }

            self.last_updated_time = self.env().block_timestamp() as u128;
            self.period_to_finish = self.env().block_timestamp() as u128 + self.reward_duration;


            self.env().emit_event(
                RewardNotified {
                    reward
                }
            );

            Ok(())
        }
        

        #[ink(message)]
        pub fn pull_out_psp22_tokens(
            &mut self,
            token: AccountId,
            amount: Balance
        ) -> Result<(), Error> {
            self.only_owner()?;
            let account = self.env().caller();
            self.transfer_from(
                self.env().account_id(),
                account,
                token,
                amount
            )?;

            Ok(())
        }


        #[ink(message)]
        pub fn set_reward_duration(
            &mut self,
            duration: Balance
        ) -> Result<(), Error> {
            self.only_owner()?;

            if self.env().block_timestamp() <= self.period_to_finish {
                return Err(Error::StakingStillInProgress)
            } // admin would not be able to update the staking duration while staking is still on going

            self.reward_duration = duration;

            self.env().emit_event(
                DurationUpdate {
                    duration
                }
            );
        }
    }
}









// function that would be created 
// VIEW 
/* 
1. total_supply , balance_of, 
2. last_time_reward_applicable
3. reward_per_token
4. earned 
5. get_reward_for_duration 
 */
// MUTATION
/* 
1. stake 
2. withdraw 
3. get_reward 
4. exit 
5. notify_reward_amount
6. pullout_any_erc_20
7. set_reward_duration
8. 




 */

/* 
Modifiers
1. only_owner
2. update_reward 
 */

/* 
THIS IS HOW TO MINT 
_instance._mint_to(_instance.env().caller(), initial_supply).expect("Should mint"); 


(
                self.reward_per_token_stored + (
                    (
                        (
                            (
                                self.last_time_reward_applicable() - self.last_updated_time
                            ) * self.reward_rate
                        ) * powf64(10.0, 18.0)
                    ) / self.total_supply
                )
            )
 */