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
        }, traits::{ZERO_ADDRESS},
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
        NotAdmin
    }



    // ===================================
    // EVENTS DECLARATION 
    // ===================================
    #[ink(event)]
    pub struct NewTPA {
        caller:AccountId,
        psp22_address:AccountId,
        lp_fee: Balance,
    }


    

    #[ink(impl)]
    impl Contract {
        // =====================================
        // MODIFIERS AND AUTHORIZATION GATE 
        // =====================================
        
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


        // ===============================
        // WRITE FUNCTIONS
        // ===============================

        // #[ink(message)]
        // pub fn stake(
        //     &self,
        //     amount: Balance
        // ) {
        //     self.reward_rate * self.reward_duration
        // }


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