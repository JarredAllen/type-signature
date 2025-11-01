//! Test the derive macro on some values.
//!
//! Part of the API for this crate is that the values don't change.

#![allow(
    dead_code,
    reason = "Fields are needed for `TypeSignature` macro, but not read or written"
)]

pub use type_signature::TypeSignature;

/// A test struct without any generics.
#[derive(TypeSignature)]
struct TestStruct {
    a: u32,
    b: String,
}

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

#[test]
fn test_derived_hashes() {
    assert_eq!(TestStruct::CONST_HASH, 0x04bd_4f41_b5bf_e0fc);

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
}
