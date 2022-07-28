use super::*;

#[allow(unused)]
use crate::Pallet as Kitties;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	transfer_ownership {
		let caller: T::AccountId = whitelisted_caller();
		let call_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));
		Kitties::<T>::set_limit_kitty(call_origin.clone(), 10);
		Kitties::<T>::create_kitty(call_origin, 0u8.into());
		let dna = OwnerToKitties::<T>::get(caller.clone()).unwrap_or(Vec::new()).pop().unwrap();
		let receiver: T::AccountId = whitelisted_caller();

	}: transfer_ownership(RawOrigin::Signed(caller), dna, receiver)

	verify {
		assert_eq!(NumberOfKitties::<T>::get(), 1);
	}

	impl_benchmark_test_suite!(Kitties, crate::mock::new_test_ext(), crate::mock::Test);
}
