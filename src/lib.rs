use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise, Timestamp, NearToken};
use near_sdk::json_types::U128;
use sha2::{Digest, Sha256};

const MIN_DEPOSIT: NearToken = NearToken::from_near(1); // 1 NEAR
const DENOMINATIONS: [NearToken; 3] = [
    NearToken::from_near(1),    // 1 NEAR
    NearToken::from_near(10),   // 10 NEAR
    NearToken::from_near(100),  // 100 NEAR
];
const MIN_DELAY: u64 = 3600 * 24; // 24 hours in seconds

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Mixer {
    // Maps commitment hash to deposit information
    deposits: LookupMap<String, Deposit>,
    // Tracks used nullifiers to prevent double-spending
    nullifiers: UnorderedSet<String>,
    // Owner of the contract
    owner: AccountId,
    // Fee percentage (in basis points, 100 = 1%)
    fee_basis_points: u16,
    // Track total deposits by denomination for analytics
    deposit_counts: LookupMap<NearToken, u64>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Deposit {
    denomination: NearToken,
    timestamp: Timestamp,
}

#[near_bindgen]
impl Mixer {
    #[init]
    pub fn new(owner: AccountId, fee_basis_points: u16) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        assert!(fee_basis_points <= 500, "Fee cannot exceed 5%");
        
        Self {
            deposits: LookupMap::new(b"d"),
            nullifiers: UnorderedSet::new(b"n"),
            owner,
            fee_basis_points,
            deposit_counts: LookupMap::new(b"c"),
        }
    }

    /// Deposits funds into the mixer pool
    #[payable]
    pub fn deposit(&mut self, commitment: String) {
        let deposit_amount = env::attached_deposit();
        
        // Validate the deposit amount is an accepted denomination
        let mut valid_denomination = false;
        for denom in DENOMINATIONS.iter() {
            if &deposit_amount == denom {
                valid_denomination = true;
                
                // Increment deposit count for this denomination
                let current_count = self.deposit_counts.get(denom).unwrap_or(0);
                self.deposit_counts.insert(denom, &(current_count + 1));
                break;
            }
        }
        assert!(valid_denomination, "Deposit must be one of the accepted denominations");
        
        // Ensure the commitment hasn't been used before
        assert!(!self.deposits.contains_key(&commitment), "Commitment already exists");
        
        // Store the commitment with deposit info
        self.deposits.insert(&commitment, &Deposit {
            denomination: deposit_amount,
            timestamp: env::block_timestamp(),
        });

        env::log_str(&format!("Deposit of {} NEAR accepted", deposit_amount.as_near()));
    }

    /// Withdraws funds from the mixer
    pub fn withdraw(&mut self, recipient: AccountId, nullifier: String, commitment: String, proof: String) {
        // Verify the nullifier hasn't been used before (prevent double spending)
        assert!(!self.nullifiers.contains(&nullifier), "Nullifier already used");

        // Verify the commitment exists
        let deposit = self.deposits.get(&commitment).expect("Commitment not found");
        
        // Verify the proof (simple version)
        assert!(self.verify_proof(&nullifier, &commitment, &proof), "Invalid proof");
        
        // Verify the time delay has passed
        let current_time = env::block_timestamp();
        assert!(current_time - deposit.timestamp >= MIN_DELAY * 1_000_000_000, 
                "Withdrawal too early, time delay not satisfied");
        
        // Mark the nullifier as used
        self.nullifiers.insert(&nullifier);
        
        // Calculate fee
        let fee = deposit.denomination.as_yoctonear() * u128::from(self.fee_basis_points) / 10000;
        let withdrawal_amount = deposit.denomination.as_yoctonear() - fee;
        
        // Transfer fee to owner
        if fee > 0 {
            Promise::new(self.owner.clone()).transfer(NearToken::from_yoctonear(fee));
        }
        
        // Transfer funds to recipient
        Promise::new(recipient.clone()).transfer(NearToken::from_yoctonear(withdrawal_amount));
        
        // Remove the commitment
        self.deposits.remove(&commitment);

        env::log_str(&format!("Withdrawal of {} NEAR processed to {}", 
            NearToken::from_yoctonear(withdrawal_amount).as_near(), recipient));
    }
    
    /// A simplified version of proof verification (not cryptographically secure)
    fn verify_proof(&self, nullifier: &str, commitment: &str, proof: &str) -> bool {
        // In a real implementation, this would be a proper cryptographic verification
        // This is a simplified placeholder that just checks format and basic consistency
        
        // Check if proof has proper length
        if proof.len() < 32 {
            return false;
        }
        
        // In a real implementation, we would verify the proof cryptographically
        // Here we're just doing a simple check to simulate proof verification
        let mut hasher = Sha256::new();
        hasher.update(nullifier);
        hasher.update(commitment);
        let result = format!("{:x}", hasher.finalize());
        
        // Check if the proof starts with the same char as the hash result
        proof.starts_with(&result[0..1])
    }
    
    /// View method to get pool stats
    pub fn get_pool_stats(&self) -> (u64, U128, Vec<(U128, u64)>) {
        let mut total_deposits = 0;
        let mut total_amount: u128 = 0;
        let mut by_denomination = Vec::new();
        
        for denom in DENOMINATIONS.iter() {
            let count = self.deposit_counts.get(denom).unwrap_or(0);
            total_deposits += count;
            total_amount += denom.as_yoctonear() * u128::from(count);
            by_denomination.push((U128(denom.as_yoctonear()), count));
        }
        
        (total_deposits, U128(total_amount), by_denomination)
    }
    
    /// Update fee (only owner)
    pub fn update_fee(&mut self, new_fee_basis_points: u16) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only owner can update fee");
        assert!(new_fee_basis_points <= 500, "Fee cannot exceed 5%");
        self.fee_basis_points = new_fee_basis_points;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain};

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    #[test]
    fn test_new() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        
        let contract = Mixer::new(accounts(1), 100);
        assert_eq!(contract.fee_basis_points, 100);
    }

    #[test]
    #[should_panic(expected = "Fee cannot exceed 5%")]
    fn test_new_excessive_fee() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        
        let _contract = Mixer::new(accounts(1), 600);
    }

    #[test]
    fn test_deposit_and_withdraw() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        
        let mut contract = Mixer::new(accounts(1), 100);
        
        // Set up deposit
        let commitment = "commitment123".to_string();
        context.attached_deposit(DENOMINATIONS[0].as_yoctonear());
        testing_env!(context.build());
        
        contract.deposit(commitment.clone());
        
        // Fast forward time
        context.block_timestamp(env::block_timestamp() + MIN_DELAY * 1_000_000_000 + 1);
        testing_env!(context.build());
        
        // Create a valid proof (simplified for test)
        let nullifier = "nullifier123".to_string();
        let mut hasher = Sha256::new();
        hasher.update(&nullifier);
        hasher.update(&commitment);
        let hash_result = format!("{:x}", hasher.finalize());
        let proof = format!("{}test", &hash_result[0..1]);
        
        // Withdraw
        contract.withdraw(accounts(2), nullifier.clone(), commitment.clone(), proof);
        
        // Verify nullifier is used
        assert!(contract.nullifiers.contains(&nullifier));
    }
}