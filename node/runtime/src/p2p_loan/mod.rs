/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_module, decl_storage, decl_event, dispatch::Result, ensure, traits::Randomness, traits::{LockableCurrency, LockIdentifier, WithdrawReason, WithdrawReasons,
	Currency, ReservableCurrency}};
use primitives::H256;
use sr_primitives::weights::SimpleDispatchInfo;
use crate::RandomnessCollectiveFlip;
use codec::{Encode, Decode};
use system::{ensure_signed};
use support::dispatch::Parameter;
use rstd::prelude::*;
 
const ID_1: LockIdentifier = *b"1       ";

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Loan<Balance: Parameter, AccountId: Parameter, BlockNumber: Parameter> {
    lender: AccountId,
    borrower: AccountId,
    amount: Balance,
    interest: Balance,
    collateral: Balance,
    period: BlockNumber,
    next_increment: BlockNumber
}

impl<Balance: Parameter, AccountId: Parameter, BlockNumber: Parameter> Loan<Balance, AccountId, BlockNumber> {
    pub fn new (lender: AccountId, borrower: AccountId, amount: Balance, interest: Balance, collateral: Balance, period: BlockNumber, next_increment: BlockNumber) -> Self {
        Loan {
            lender,
            borrower,
            amount,
            interest,
            collateral,
            period,
            next_increment
        }
    }

    pub fn approve(&mut self, lender: AccountId) {
        self.lender = lender;
    }

    pub fn approver_is_lender(&self, lender: AccountId) -> bool {
        self.lender == lender && self.borrower != lender   
    }

    pub fn redeemer_is_borrower(&self, borrower: AccountId) -> bool {
        self.borrower == borrower && self.lender != borrower
    }
}

// Module's function and Methods of custom struct to be placed here
impl<T: Trait> Module<T> {

    pub fn activate_loan(mut loan: Loan<T::Balance, T::AccountId, T::BlockNumber>, current: T::BlockNumber) -> Loan<T::Balance, T::AccountId, T::BlockNumber> {
        // (1 + interest/ 1000) * <loan amount>
        loan.amount += (loan.amount * loan.interest) / Self::to_balance(1, "kilo");
        loan.next_increment = current + loan.period;
        loan
    }

    pub fn process_loan(loan_id: H256, current_block: T::BlockNumber) -> Result {
        let mut updated_loan = Self::loan(loan_id.clone());
        let before = updated_loan.amount;
        updated_loan = Self::activate_loan(updated_loan.clone(), current_block);
        <Loans<T>>::mutate(loan_id.clone(), |l| {*l = updated_loan.clone()});
        let next = updated_loan.clone().next_increment;
        if <LoanCallBacks<T>>::exists(next.clone()) {
            <LoanCallBacks<T>>::mutate(next.clone(), |c| {c.push(loan_id);});
            Self::deposit_event(RawEvent::LoanAmountIncreased(loan_id.clone(), before, updated_loan.clone().amount, next.clone()));
        } else {
            <LoanCallBacks<T>>::insert(next.clone(), vec!{loan_id});
            Self::deposit_event(RawEvent::LoanAmountIncreased(loan_id.clone(), before, updated_loan.clone().amount, next.clone()));    
        }
        return Ok(());
	}
	
	pub fn to_balance(u: u32, digit: &str) -> T::Balance {
		let power = |u: u32, p: u32| -> T::Balance {
			let mut base = T::Balance::from(u);
			for _i in 0..p { 
				base *= T::Balance::from(10)
			}
			return base;
		};
		let result = match digit  {
			"femto" => T::Balance::from(u),
			"nano" =>  power(u, 3),
			"micro" => power(u, 6),
			"milli" => power(u, 9),
			"one" => power(u,12),
			"kilo" => power(u, 15),
			"mega" => power(u, 18),
			"giga" => power(u, 21),
			"tera" => power(u, 24),
			"peta" => power(u, 27),
			"exa" => power(u, 30),
			"zetta" => power(u, 33),
			"yotta" => power(u, 36),
			_ => T::Balance::from(u)
		}; 
		result 
    }
    

}

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as P2PLoan {
        pub Loans get(loan): map H256 => Loan<T::Balance, T::AccountId, T::BlockNumber>;
        pub LoanCallBacks get(callback): map T::BlockNumber => Vec<H256>;
    }
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
        fn deposit_event() = default;
        
        fn on_finalize(block_number: T::BlockNumber) {
			let loans = Self::callback(block_number);
			for i in loans {
				if let Err(e) = Self::process_loan(i, block_number) {
					sr_primitives::print(e);
				}
			}
		}

        #[weight = SimpleDispatchInfo::FixedNormal(0)]
        pub fn request_loan(origin, lender: T::AccountId, amount: T::Balance, interest: T::Balance, collateral: T::Balance, period: T::BlockNumber) -> Result {
            let borrower = ensure_signed(origin)?;
            let new_loan = Loan::new(lender, borrower, amount, interest.clone(), collateral, period, T::BlockNumber::from(0));
            let loan_hash = H256::from_slice(&RandomnessCollectiveFlip::random_seed().encode() as &[u8]);
            ensure!(!<Loans<T>>::exists(loan_hash.clone()), "Hash collision!");
            <Loans<T>>::insert(loan_hash, new_loan.clone());
            Self::deposit_event(RawEvent::LoanRequested(new_loan.clone().lender, new_loan.clone().borrower, loan_hash.clone()));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::FixedNormal(0)]
        pub fn approve_loan(origin, loan_id: H256) -> Result {
            let approver = ensure_signed(origin)?;
            ensure!(<Loans<T>>::exists(loan_id.clone()), "Loan does not exist");
            let mut the_loan = Self::loan(loan_id.clone());
            ensure!(the_loan.approver_is_lender(approver.clone()), "You are not the lender for this borrower");
            the_loan.approve(approver);
            let current_block = <system::Module<T>>::block_number();
            the_loan = Self::activate_loan(the_loan, current_block);
            <Loans<T>>::mutate(loan_id, |l| {*l = the_loan.clone()});
            <balances::Module<T>>::set_lock(ID_1, &the_loan.clone().borrower, the_loan.clone().collateral,  T::BlockNumber::from(100000000 as u64), WithdrawReasons::all());
            Self::process_loan(loan_id.clone(), current_block.clone()).expect("loan operation is stored in a callback storage");
            Self::deposit_event(RawEvent::LoanApproved(the_loan.clone().lender, the_loan.clone().borrower, current_block.clone()));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::FixedNormal(0)]
        pub fn redeem(origin, loan_id: H256) -> Result {
            let borrower = ensure_signed(origin)?;
            ensure!(<Loans<T>>::exists(loan_id.clone()), "Loan does not exist");
            let the_loan = Self::loan(loan_id.clone());
            ensure!(the_loan.redeemer_is_borrower(borrower), "You are not the redeemer for this loan");
            <balances::Module<T>>::remove_lock(ID_1, &the_loan.clone().borrower);
            <balances::Module<T> as Currency<_>>::transfer(&the_loan.clone().borrower, &the_loan.clone().lender, the_loan.clone().amount).expect("Transfer the owed amount from borrower to lender");
            <Loans<T>>::remove(loan_id);
            Self::deposit_event(RawEvent::LoanRedeemed(the_loan.clone().lender, the_loan.clone().borrower, the_loan.clone().amount));
            Ok(())
        }

        // TODO add liquidate()
        // I do not know what derivate to make with this loan 

	}
}


decl_event!(
	pub enum Event<T> where Time = <T as system::Trait>::BlockNumber, Lender  = <T as system::Trait>::AccountId, Borrower = <T as system::Trait>::AccountId, Redeemed = <T as balances::Trait>::Balance, Before = <T as balances::Trait>::Balance, After = <T as balances::Trait>::Balance {
        LoanRequested(Lender, Borrower, H256),
        LoanApproved(Lender, Borrower, Time),
        LoanRedeemed(Lender, Borrower, Redeemed),
        LoanAmountIncreased(H256, Before, After, Time),
	}
);
