/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, dispatch::Result, Parameter, ensure};
//use primitives::RuntimeDebug;
use codec::{Encode, Decode};
use system::ensure_signed;
use rstd::prelude::*;
use rstd::collections::btree_map::{BTreeMap};
use crate::access;

/// Data storage type for each account
#[derive(Encode, Decode, Default, Clone, Eq, PartialEq)]
pub struct Data {
	pub extension_id: u32,
	pub bytes: Vec<u8>
}

impl Data {
	pub fn new (extension_id: u32, bytes: Vec<u8>) -> Self {
		Data {
			extension_id,
			bytes
		}
	}
}

/// The module's configuration trait.
pub trait Trait: system::Trait + access::Trait {

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// Dazta storage type for each account
#[derive(Encode, Decode, Default, Clone, Eq, PartialEq)]
pub struct PersonalStorage<AccountId: Parameter> {
	stack: BTreeMap<Vec<u8>, Vec<Data>>,
	remove_key: AccountId
}

impl <AccountId: Parameter> PersonalStorage<AccountId> {
	pub fn new (remove_key: AccountId) -> Self {
        PersonalStorage {
            stack: BTreeMap::<Vec<u8>, Vec<Data>>::new(),
            remove_key
        }
	}
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Storage {
		Storages get(storage): map Vec<u8> => PersonalStorage<T::AccountId>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event() = default;

		pub fn register_storage(origin, id: Vec<u8>, remove_key: T::AccountId) -> Result {
			let who = ensure_signed(origin)?;
			ensure!(access::Module::<T>::is_id_owner(id.clone(), who.clone()), "You are not the owner of this id");
			let new_strorage = PersonalStorage::new(remove_key);
			ensure!(!<Storages<T>>::exists(id.clone()), "The storage already exists");
			<Storages<T>>::insert(id.clone(), new_strorage);
			Self::deposit_event(RawEvent::StorageRegistered(who, id));
			Ok(())
		}

		pub fn store(origin, sender: Vec<u8>, receiver: Vec<u8>, extension_id: u32, buffer: Vec<u8>) -> Result {
			let who = ensure_signed(origin)?;
			ensure!(access::Module::<T>::is_id_owner(sender.clone(), who), "You are not the owner of sender id");
			let result = match <access::Statuses>::get((receiver.clone(), sender.clone())) {
				Some(x) if x == 1 => {
					ensure!(<Storages<T>>::exists(receiver.clone()), "The receiver has not setup the storage for his id yet");
					let mut receiver_storage = Self::storage(receiver.clone());
					let data = Data::new(extension_id, buffer);
					if receiver_storage.stack.contains_key(&sender.clone()) {
						let mut data_vec = receiver_storage.stack[&sender.clone()].clone().to_vec();
						data_vec.push(data);
						receiver_storage.stack.remove(&sender.clone());
						receiver_storage.stack.insert(sender.clone(), data_vec);
					} else {
						receiver_storage.stack.insert(sender.clone(), vec![data]);
					}
					<Storages<T>>::mutate(receiver.clone(), |s| *s = receiver_storage);
					Self::deposit_event(RawEvent::DataStored(sender));
					Ok(())
				},
				Some(x) if x == 0 => Err("You are blocked by the sender"),
				None => Err("You are not registered in access list. Are you a spammer?"),
				_ => Err("Not implemented yet"),
			}; 
			result 
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		StorageRegistered(AccountId, Vec<u8>),
		DataStored(Vec<u8>),
		DataRemoved(Vec<u8>),
	}
);

