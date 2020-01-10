/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure};
use primitives::H256;
use sr_primitives::weights::SimpleDispatchInfo;
use codec::{Encode, Decode};
use system::{ensure_signed};
use rstd::prelude::*;
use crate::did;

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
        pub fn propose(origin, content_id: Vec<u8>, proposer_id: Vec<u8>, proposer_id_signature: Vec<u8>) -> Result {
            let proposer = ensure_signed(origin)?;
            ensure!(<did::IDs>::exists(proposer_id.clone()), "The proposer has not made did yet");
            ensure!(did::Module::<T>::is_id_owner(proposer_id.clone(), proposer.clone()), "Proposer does not own this DID");
            
            let phys_contract = PhysContract::new(content_id, proposer_id.clone(), proposer_id_signature);
            <PhysContracts>::insert(proposer_id.clone(), phys_contract);
            Self::deposit_event(RawEvent::PhysContractProposed(proposer_id, proposer));
            Ok(())
        }

        pub fn approve(origin, proposer_id: Vec<u8>, approver_id: Vec<u8>, approver_id_signature: Vec<u8>) -> Result {
            let approver = ensure_signed(origin)?;
            ensure!(<did::IDs>::exists(approver_id.clone()), "The approver has not made did yet");
            ensure!(did::Module::<T>::is_id_owner(approver_id.clone(), approver.clone()), "Approver does not own this DID");
            
            let mut new_phys_contract = Self::contract(proposer_id.clone());
            new_phys_contract.approver_id = approver_id.clone();
            new_phys_contract.approver_signature = approver_id_signature;
            <PhysContracts>::mutate(proposer_id.clone(), |c| *c = new_phys_contract.clone());
            <PhysContracts>::insert(approver_id.clone(), new_phys_contract);
            
            Self::deposit_event(RawEvent::PhysContractApproved(approver_id, approver));
            Ok(())
        }

        /*
        #[weight = SimpleDispatchInfo::FixedNormal(0)]
        pub fn remove(origin, proposer_id: Vec<u8>, approver_id: Vec<u8>) -> Result {
            let proposer = ensure_signed(origin)?;
            ensure!(<IDs>::exists(id.clone()), "The proposer id does not exist");
            let proposer_id = Self::id(id.clone());
            let proposer_hash = H256::from_slice(&issuer.encode());
            ensure!(proposer_id.owner == proposer_hash, "You are not the owner of this identity");
            <PhysContracts>::remove(proposer_id.clone());
            if <PhysContract>::exists()
            Self::deposit_event(RawEvent::PhysContractRemoved(proposer_id, proposer));
            Ok(())
        }
        */

        /*
        #[weight = SimpleDispatchInfo::FixedNormal(0)]
        pub fn update(origin, id: Vec<u8>, content_hash: Vec<u8>) -> Result {

            let proposer = ensure_signed(origin)?;
            let proposer_hash = H256::from_slice(&proposer.encode());
            ensure!(did::<IDs>::exists(id.clone()), "The proposer has not made did yet");
            ensure!(did::is_id_owner(proposer_id, proposer), "Proposer does not own this DID");
            
            // Update DID 
            let contract_claimer = DID::new(public_key.clone(), issuer_hash.clone());
            <IDs>::mutate(id.clone(), |a| *a = did_claimer);
            Self::deposit_event(RawEvent::IdChanged(id, public_key, issuer));
            Ok(())
        }
        */
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
