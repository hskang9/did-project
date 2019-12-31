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
pub struct PersonalAccess {
	pub access_list: Vec<Vec<u8>>,
    /// hash from root identity public key
    pub root: H256
}

impl PersonalAccess {
    pub fn new (id: Vec<u8>, root: H256) -> Self {
        PersonalAccess{
            access_list: vec![id],
            root
        }
    }

    pub fn add_id(mut self, id: Vec<u8>) -> Self {
        self.access_list.push(id);
        self
    }

    pub fn remove_id(mut self, id:Vec<u8>) -> Self {
        let index = self.access_list.iter().position(|x| **x == *id).unwrap();
        self.access_list.remove(index);
        self
    }
}

// Module's function and Methods of custom struct to be placed here
impl<T: Trait> Module<T> {
    pub fn is_id_owner(id:Vec<u8>, sender: T::AccountId) -> bool {
        let access_who = Self::access(id.clone());
        let root_hash = H256::from_slice(&sender.encode());
        access_who.root == root_hash    
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
	trait Store for Module<T: Trait> as AccessModule {
        pub Accesses get(access): map Vec<u8> => PersonalAccess;
        pub Statuses get(status): map (Vec<u8>, Vec<u8>) => Option<u32>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

         // Grant access from receiver to sender to get encrypted message
         pub fn grant_access(origin, id: Vec<u8>, sender: Vec<u8>) -> Result {
            let who = ensure_signed(origin)?;
            ensure!(<Accesses>::exists(id.clone()), "You have not registered your identity for data exchange");
            let access_who = Self::access(id.clone());
            let root_hash = H256::from_slice(&who.encode());
            ensure!(access_who.root == root_hash, "You do not own this identity");
            
            // Grant access to sender to store encrypted message to your storage
            <Accesses>::mutate(id.clone(), |a| *a = access_who.add_id(sender.clone()));
            <Statuses>::mutate((id.clone(), sender.clone()), |a| *a = core::prelude::v1::Some(1));
            Self::deposit_event(RawEvent::AccessGranted(id, sender));
            Ok(())
        }

        pub fn remove_access(origin, id: Vec<u8>, sender: Vec<u8>) -> Result {
            let who = ensure_signed(origin)?;
            ensure!(<Accesses>::exists(id.clone()), "You have not registered your identity for data exchange");
            let access_who = Self::access(id.clone());
            let root_hash = H256::from_slice(&who.encode());
            ensure!(access_who.root == root_hash, "You do not own this identity");

            // Remove access to sender to store encrypted message to your storage
            <Accesses>::mutate(id.clone(), |a| *a = access_who.remove_id(sender.clone()));
            <Statuses>::remove((id.clone(), sender.clone()));
            Self::deposit_event(RawEvent::AccessRemoved(id, sender));
            Ok(())
        }

        pub fn block(origin, id: Vec<u8>, sender: Vec<u8>) -> Result {
            let who = ensure_signed(origin)?;
            ensure!(<Accesses>::exists(id.clone()), "You have not registered your identity for data exchange");
            let access_who = Self::access(id.clone());
            let root_hash = H256::from_slice(&who.encode());
            ensure!(access_who.root == root_hash, "You do not own this identity");
            
            // Block access to sender to store encrypted message to your storage
            <Statuses>::mutate((id.clone(), sender.clone()), |a| *a = core::prelude::v1::Some(0));
            Self::deposit_event(RawEvent::AccessBlocked(id, sender));
            Ok(())
        }

        pub fn register(origin, id: Vec<u8>) -> Result {
            let who = ensure_signed(origin)?;
            let root_hash = H256::from_slice(&who.encode());
            ensure!(!<Accesses>::exists(id.clone()), "The id is already registered");
            
            let access_who = PersonalAccess::new(id.clone(), root_hash.clone());
            <Accesses>::insert(id.clone(), access_who);
            <Statuses>::insert((id.clone(), id.clone()), 1);
            Self::deposit_event(RawEvent::IdRegistered(id, who));
            Ok(())
        }
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
        AccessGranted(Vec<u8>, Vec<u8>),
        AccessBlocked(Vec<u8>, Vec<u8>),
        AccessRemoved(Vec<u8>, Vec<u8>),
        IdRegistered(Vec<u8>, AccountId),
        TempAccess(Vec<u8>, Vec<u8>, u64),
	}
);
