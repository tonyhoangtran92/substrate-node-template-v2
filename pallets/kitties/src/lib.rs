#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;
use frame_support::{dispatch::fmt, inherent::Vec, pallet_prelude::*};
use frame_system::pallet_prelude::*;

use frame_support::traits::{Currency, Get};
use frame_support::{traits::Randomness};
use frame_support::sp_runtime::traits::{Hash};

use pallet_timestamp::{self as timestamp};
use pallet_kitty_limit::KittyLimit;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	pub use super::*;
	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: T::Hash,
		price: BalanceOf<T>,
		gender: Gender,
		account: T::AccountId,
		created_date: <T as pallet_timestamp::Config>::Moment
	}

	impl<T: Config> fmt::Debug for Kitty<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Kitty")
			 .field("dna", &self.dna)
			 .field("price", &self.price)
			 .field("gender", &self.gender)
			 .field("account", &self.account)
			 .field("created_date", &self.created_date)
			 .finish()
		}
	}

	#[derive(TypeInfo, Encode, Decode, Debug)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender {
		fn default() -> Self {
			Gender::Male
		}
	}
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
		type KittyLimit: KittyLimit;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn student_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type NumberOfKitties<T> = StorageValue<_, u32, ValueQuery>;
	pub type NewOwner<T: Config> = T::AccountId;

	// key : dna
	//value : kitty
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner_to_kitties)]
	pub(super) type OwnerToKitties<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::Hash> , OptionQuery>;

	#[pallet::storage]
    #[pallet::getter(fn get_nonce)]
    pub(super) type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyStored(T::Hash, BalanceOf<T>),
		TransferKittySuccess(T::Hash, NewOwner<T>),
		SetLimitKittySuccess
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		TooShort,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		KittyNotFound,
		NotOwner,
		OverKittyLimit,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	//extrinsic
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, price: BalanceOf<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// generate random hash
			let dna = Self::random_hash(&who);

			// ensure!(dna.len() > 10, Error::<T>::TooShort);
			let gender = Self::gen_gender(dna)?;
			let _now = <timestamp::Pallet<T>>::get();
			let kitty = Kitty { dna: dna, price, gender, account: who.clone(), created_date: _now };
			log::warn!("create_kitty: {:?}", kitty);

			let mut kitties = <OwnerToKitties<T>>::get(who.clone()).unwrap_or(Vec::new());
			ensure!(kitties.len() + 1 <= T::KittyLimit::get().try_into().unwrap_or(1000), Error::<T>::OverKittyLimit);
			kitties.push(dna);


			<OwnerToKitties<T>>::insert(who.clone(), kitties);

			let mut number_of_kitties = <NumberOfKitties<T>>::get();
			<Kitties<T>>::insert(dna.clone(), kitty);
			number_of_kitties += 1;
			NumberOfKitties::<T>::put(number_of_kitties);

			let _nonce = Self::increment_nonce();
			// Emit an event.
			Self::deposit_event(Event::KittyStored(dna, price));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(23_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_ownership(origin: OriginFor<T>, dna: T::Hash, new_owner: NewOwner<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty = <Kitties<T>>::get(dna);
			ensure!(kitty.is_some(), Error::<T>::KittyNotFound);
			let mut kitty = kitty.unwrap();
			ensure!(kitty.account == who, Error::<T>::NotOwner);


			// Assign new owner for kitty
			kitty.account = new_owner.clone();
			<Kitties<T>>::insert(dna, kitty);

			// Update number of kitties owned by new owner
			let mut kitties = <OwnerToKitties<T>>::get(new_owner.clone()).unwrap_or(Vec::new());
			kitties.push(dna);

			<OwnerToKitties<T>>::insert(new_owner.clone(), kitties);

			// Remove kitty from old owner
			let mut kitties = <OwnerToKitties<T>>::get(who.clone()).unwrap_or(Vec::new());
			kitties.retain(|x| x != &dna);
			<OwnerToKitties<T>>::insert(who.clone(), kitties);

			// Emit an event.
			Self::deposit_event(Event::TransferKittySuccess(dna, new_owner));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_limit_kitty(origin: OriginFor<T>, value: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let _limit = T::KittyLimit::set(value);
			// Emit an event.
			Self::deposit_event(Event::SetLimitKittySuccess);

			Ok(())
		}
	}

	// Helper function
	impl<T: Config> Pallet<T> {
		fn gen_gender(dna: T::Hash) -> Result<Gender, Error<T>> {
			let mut res = Gender::Male;
			if dna.as_ref()[0] % 2 == 0 {
				res = Gender::Female;
			}
			Ok(res)
		}

		fn increment_nonce() -> DispatchResult {
			<Nonce<T>>::try_mutate(|nonce| {
				let next = nonce.checked_add(1).ok_or("Overflow")?; // TODO Part III: Add error handling
				*nonce = next;

				Ok(().into())
			})
		}

		fn random_hash(sender: &T::AccountId) -> T::Hash {
            let nonce = <Nonce<T>>::get();
            let seed = T::KittyRandomness::random_seed();

            T::Hashing::hash_of(&(seed, &sender, nonce))
        }
	}
}
