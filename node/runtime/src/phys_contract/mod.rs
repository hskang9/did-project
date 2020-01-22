/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure, traits::Randomness};
use primitives::H256;
use sr_primitives::weights::SimpleDispatchInfo;
use codec::{Encode, Decode};
use system::{ensure_signed};
use rstd::prelude::*;
use crate::did;
use crate::RandomnessCollectiveFlip;

/*
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub enum Status {
    /// Allowing access
    Yes,
    /// Blocked
    Blocked,
    /// Temporary
    Temporary(u64)
}

impl Default for Status {
    fn default() -> Self { Status::Yes }
}
*/

//pub type Id = Vec<u8>;

/// Data storage type for each account
#[derive(Encode, Decode, Default, Clone, Eq, PartialEq)]
pub struct PhysContract {
    pub content_id: Vec<u8>,
    pub proposer_id: Vec<u8>,
    pub approver_id: Vec<u8>,
    pub proposer_signature: Vec<u8>,
    pub approver_signature: Vec<u8>
}

impl PhysContract {
    pub fn new (content_id: Vec<u8>, proposer_id: Vec<u8>, proposer_signature: Vec<u8>) -> Self {
        PhysContract {
            content_id,
            proposer_id,
            approver_id: vec!{0},
            proposer_signature,
            approver_signature: vec!{0}
        }
    }
}

// Module's function and Methods of custom struct to be placed here
impl<T: Trait> Module<T> {
    
}

/// The module's configuration trait.
pub trait Trait: system::Trait +  did::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as PhysContract {
        pub PhysContracts get(contract): map Vec<u8> => PhysContract;
    }
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FixedNormal(0)]
        pub fn propose(origin, content_id: Vec<u8>, proposer_id: Vec<u8>, approver_id: Vec<u8>, proposer_id_signature: Vec<u8>) -> Result {
            let proposer = ensure_signed(origin)?;
            ensure!(<did::IDs>::exists(proposer_id.clone()), "The proposer has not made did yet");
            ensure!(did::Module::<T>::is_id_owner(proposer_id.clone(), proposer.clone()), "Proposer does not own this DID");
            let contract_id = H256::from_slice(&[&proposer_id.clone().encode() as &[u8], &RandomnessCollectiveFlip::random_seed().encode() as &[u8]].concat());

            let phys_contract = PhysContract::new(content_id, proposer_id.clone(), approver_id.clone(), proposer_id_signature);
            <PhysContracts>::insert(contract_id.as_bytes().to_vec(), phys_contract);
            Self::deposit_event(RawEvent::PhysContractProposed(contract_id.as_bytes().to_vec(), proposer));
            Ok(())
        }

        pub fn approve(origin, contract_id: Vec<u8>) -> Result {
            let approver = ensure_signed(origin)?;
            let phys_contract = Self::contract(contract_id);
            let approver_id = phys_contract.approver_id;
            ensure!(<did::IDs>::exists(approver_id.clone()), "The approver's did does not exist");
            ensure!(did::Module::<T>::is_id_owner(approver_id.clone(), approver.clone()), "Approver does not own this DID");
            

            let mut new_phys_contract = phys_contract.clone();
            new_phys_contract.approver_signature = approver_id_signature;
            <PhysContracts>::mutate(contract_id.clone(), |c| *c = new_phys_contract.clone());
            
            Self::deposit_event(RawEvent::PhysContractApproved(contract_id, approver_id));
            Ok(())
        }

        
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        PhysContractProposed(Vec<u8>, AccountId),
        PhysContractApproved(Vec<u8>, AccountId),
        PhysContractRemoved(Vec<u8>, AccountId),
        PhysContractChanged(Vec<u8>, Vec<u8>, AccountId),
	}
);
