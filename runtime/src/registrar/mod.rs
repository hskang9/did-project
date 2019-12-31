/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure};
use primitives::H256;
use codec::{Encode, Decode};
use system::{ensure_signed};
use rstd::prelude::*;

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
pub struct DID {
	pub public_key: Vec<u8>,
    /// hash from issuer identity public key
    pub issuer: H256
}

impl DID {
    pub fn new (public_key: Vec<u8>, issuer: H256) -> Self {
        DID {
            public_key,
            issuer
        }
    }
}

// Module's function and Methods of custom struct to be placed here
impl<T: Trait> Module<T> {
    pub fn is_id_issuer(id:Vec<u8>, issuer: T::AccountId) -> bool {
        let access_who = Self::id(id.clone());
        let root_hash = H256::from_slice(&issuer.encode());
        access_who.issuer == root_hash    
    }
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as DID {
        pub Registrars get(id): map Vec<u8> => DID;
    }
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

        pub fn register(origin, id: Vec<u8>, public_key: Vec<u8>) -> Result {
            let issuer = ensure_signed(origin)?;
            let issuer_hash = H256::from_slice(&issuer.encode());
            ensure!(!<IDs>::exists(id.clone()), "The id is already issued");
            
            let DID_claimer = DID::new(id.clone(), issuer_hash.clone());
            <IDs>::insert(id.clone(), DID_claimer);
            Self::deposit_event(RawEvent::IdIssued(id, issuer));
            Ok(())
        }

        pub fn remove(origin, id: Vec<u8>) -> Result {
            let issuer = ensure_signed(origin)?;
            let root_hash = H256::from_slice(&issuer.encode());
            ensure!(<IDs>::exists(id.clone()), "The id does not exist");
            let DID_claimer = Self::id(id.clone());
            let issuer_hash = H256::from_slice(&issuer.encode());
            ensure!(DID_claimer.issuer == issuer_hash, "You are not the issuer of this identity");
            <IDs>::remove(id.clone());
            Self::deposit_event(RawEvent::IdRemoved(id, issuer));
            Ok(())
        }

        pub fn block(origin, id: Vec<u8>, public_key: Vec<u8>) -> Result {
            let issuer = ensure_signed(origin)?;
            ensure!(<IDs>::exists(id.clone()), "DID is not registered");
            let DID_claimer = Self::id(id.clone());
            let issuer_hash = H256::from_slice(&issuer.encode());
            ensure!(DID_claimer.issuer == issuer_hash, "You are not the issuer of this identity");
            
            // Update DID 
            let DID_claimer = DID::new(public_key.clone(), issuer_hash.clone());
            <IDs>::mutate(id.clone(), |a| *a = DID_claimer);
            Self::deposit_event(RawEvent::IdChanged(id, public_key, issuer));
            Ok(())
        }
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        IdIssued(Vec<u8>, AccountId),
        IdRemoved(Vec<u8>, AccountId),
        IdChanged(Vec<u8>, Vec<u8>, AccountId),
	}
);
