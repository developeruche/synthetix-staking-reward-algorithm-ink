# Staking Reward Smart contract for PSP22 token 

## This staking algorithm models the Synthetix staking smart contract logic 

### function that would be created 
- VIEW FUNCTIONS 
1. total_supply , balance_of, 
2. last_time_reward_applicable
3. reward_per_token
4. earned 
5. get_reward_for_duration 

- MUTATION FUNCTION 
1. stake 
2. withdraw 
3. get_reward 
4. exit 
5. notify_reward_amount
6. pullout_any_erc_20
7. set_reward_duration

- MODIFIERS 
1. only_owner
2. update_reward 