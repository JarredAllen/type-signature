//! Test the derive macro on some values.
//!
//! Part of the API for this crate is that the values don't change.

#![allow(
    dead_code,
    reason = "Fields are needed for `TypeSignature` macro, but not read or written"
)]

pub use type_signature::TypeSignature;

#[derive(TypeSignature)]
struct TestUnit;

/// A test struct without any generics.
#[derive(TypeSignature)]
struct TestStruct {
    a: u32,
    b: String,
}

/// A test tuple struct without any generics.
#[derive(TypeSignature)]
struct TestTupleStruct(u32, String);

/// A test struct with some generics.
#[derive(TypeSignature)]
struct TestStructGeneric<Ty> {
    a: Ty,
    b: String,
}

/// A test struct with some generics.
#[derive(TypeSignature)]
struct TestStructGenericConstrained<Ty: PartialEq> {
    a: Ty,
    b: String,
}

/// A test struct without any generics.
#[derive(TypeSignature)]
struct TestStructConstGeneric<const COUNT: usize> {
    a: u32,
    b: [String; COUNT],
}

#[derive(TypeSignature)]
struct TestGenericBool<const BOOL: bool> {
    a: i8,
}

#[derive(TypeSignature)]
enum TestEnum {
    A(u32, i32),
    B { a: u32, b: String },
}

#[derive(TypeSignature)]
enum TestEnumGeneric<const LENGTH: usize, T> {
    A(T, [i32; LENGTH]),
    B { a: T, b: [String; LENGTH] },
}

#[derive(TypeSignature)]
union TestUnion {
    a: u32,
    b: f64,
}

#[derive(TypeSignature)]
union TestUnionGeneric<T: Copy> {
    a: T,
    b: u64,
}

#[test]
fn test_derived_hashes() {
    assert_eq!(TestUnit::CONST_HASH, 0xe8b2_2977_f5ae_fe94);
    assert_eq!(TestStruct::CONST_HASH, 0x131a_87bc_de0f_eae9);
    assert_eq!(TestTupleStruct::CONST_HASH, 0xba5c_6efb_76b8_e97d);

    assert_eq!(TestStructGeneric::<u32>::CONST_HASH, 0xf36b_5b81_3448_e6bd);
    assert_eq!(TestStructGeneric::<i32>::CONST_HASH, 0x5311_e941_e5e1_317d);
    assert_eq!(
        TestStructGeneric::<TestStruct>::CONST_HASH,
        0x315f_a4b8_433b_6f3d,
    );
    assert_eq!(
        TestStructConstGeneric::<8>::CONST_HASH,
        0x5772_f58d_8002_0f26,
    );
    assert_eq!(
        TestStructConstGeneric::<9>::CONST_HASH,
        0x22ed_6918_75dc_c50e,
    );

    assert_eq!(TestGenericBool::<false>::CONST_HASH, 0x64a9_4738_2a48_179f);
    assert_eq!(TestGenericBool::<true>::CONST_HASH, 0x8380_00d7_4ddd_715d);

    assert_eq!(TestEnum::CONST_HASH, 0xf0ab_1786_fd46_2e0f);
    assert_eq!(TestEnumGeneric::<1, u32>::CONST_HASH, 0xd865_ec03_7b63_268d);
    assert_eq!(TestEnumGeneric::<2, u32>::CONST_HASH, 0x440a_1b42_6910_fd7a);
    assert_eq!(
        TestEnumGeneric::<1, TestUnit>::CONST_HASH,
        0x949b_e7d4_8683_2e9d,
    );

    assert_eq!(TestUnion::CONST_HASH, 0x376b_05ee_d2f0_7a50);
    assert_eq!(TestUnionGeneric::<u32>::CONST_HASH, 0xa993_b675_3aa0_a3d3);
    assert_eq!(TestUnionGeneric::<i64>::CONST_HASH, 0xa988_d718_6c15_4897);
}

#[test]
fn test_const_hash_computation() {
    #[track_caller]
    fn assert<T: TypeSignature>() {
        assert_eq!(T::CONST_HASH, T::SIGNATURE.const_hash());
    }
    assert::<TestUnit>();
    assert::<TestStruct>();
    assert::<TestStructGeneric<u32>>();
    assert::<TestStructGenericConstrained<String>>();
    assert::<TestEnumGeneric<5, u32>>();
    assert::<TestUnion>();
    assert::<TestUnionGeneric<u32>>();
}
