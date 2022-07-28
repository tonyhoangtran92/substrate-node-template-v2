use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		assert_ok!(DemoModule::create_student(Origin::signed(1), "LocDTLocDT".as_bytes().to_vec(), 23));
		assert_eq!(DemoModule::student_id(), 1);
	});
}

#[test]
fn create_student_with_incorrect_info() {
	new_test_ext().execute_with(|| {
		assert_noop!(DemoModule::create_student(Origin::signed(1), "LocDT".as_bytes().to_vec(), 23 ), Error::<Test>::TooShort);
		assert_noop!(DemoModule::create_student(Origin::signed(1), "LocDTLocDT".as_bytes().to_vec(), 11 ), Error::<Test>::TooYoung);
	});
}


#[test]
fn create_student_with_correct_info() {
	new_test_ext().execute_with(|| {
		assert_ok!(DemoModule::create_student(Origin::signed(1), "LocDTLocDT".as_bytes().to_vec(), 23));
		assert_eq!(DemoModule::student(0).unwrap().name,  "LocDTLocDT".as_bytes().to_vec());
		assert_eq!(DemoModule::student(0).unwrap().age, 23);
		assert_eq!(DemoModule::student(0).unwrap().gender, DemoModule::gen_gender("LocDTLocDT".as_bytes().to_vec()).unwrap());
	});
}