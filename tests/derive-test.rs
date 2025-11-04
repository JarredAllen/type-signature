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

/// A test struct without any generics.
#[derive(TypeSignature)]
struct TestStructConstGeneric<const COUNT: usize> {
    a: u32,
    b: [String; COUNT],
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

#[test]
fn test_derived_hashes() {
    assert_eq!(TestUnit::CONST_HASH, 0x516b_1ca0_0731_3421);
    assert_eq!(TestStruct::CONST_HASH, 0x04bd_4f41_b5bf_e0fc);
    assert_eq!(TestTupleStruct::CONST_HASH, 0x1605_8b3c_0bfa_5e6c);

    assert_eq!(TestStructGeneric::<u32>::CONST_HASH, 0x8a4a_b9db_71d8_5be7);
    assert_eq!(TestStructGeneric::<i32>::CONST_HASH, 0x3516_90b2_fa95_346f);
    assert_eq!(
        TestStructGeneric::<TestStruct>::CONST_HASH,
        0xbe19_a890_a5f5_576f,
    );
    assert_eq!(
        TestStructConstGeneric::<8>::CONST_HASH,
        0x5f16_a7b7_c443_9080,
    );
    assert_eq!(
        TestStructConstGeneric::<9>::CONST_HASH,
        0x46db_a403_2ea4_ca02,
    );

    assert_eq!(TestEnum::CONST_HASH, 0xc841_1fb4_bb52_c6bb);
    assert_eq!(TestEnumGeneric::<1, u32>::CONST_HASH, 0xc1b1_2cba_8758_b283);
    assert_eq!(TestEnumGeneric::<2, u32>::CONST_HASH, 0x6609_913c_646d_e3e0);
    assert_eq!(
        TestEnumGeneric::<1, TestUnit>::CONST_HASH,
        0x7dfa_64be_80cd_776a,
    );
}
