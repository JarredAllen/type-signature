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
    assert_eq!(TestUnit::CONST_HASH, 0x2446_9e8c_0e4e_3d4c);
    assert_eq!(TestStruct::CONST_HASH, 0x7068_f6f1_a597_2995);
    assert_eq!(TestTupleStruct::CONST_HASH, 0x511d_d71f_5726_c6e0);

    assert_eq!(TestStructGeneric::<u32>::CONST_HASH, 0x5f58_0797_60df_cafc);
    assert_eq!(TestStructGeneric::<i32>::CONST_HASH, 0x3831_4104_ac5b_860c);
    assert_eq!(
        TestStructGeneric::<TestStruct>::CONST_HASH,
        0xde92_ebad_566f_94d8,
    );
    assert_eq!(
        TestStructConstGeneric::<8>::CONST_HASH,
        0xb636_e030_0648_3dde,
    );
    assert_eq!(
        TestStructConstGeneric::<9>::CONST_HASH,
        0xe44e_0414_f6b9_0c72,
    );

    assert_eq!(TestGenericBool::<false>::CONST_HASH, 0xbc52_f29c_2410_f176);
    assert_eq!(TestGenericBool::<true>::CONST_HASH, 0x1379_d224_4310_812c);

    assert_eq!(TestEnum::CONST_HASH, 0xc5c4_3b76_21f8_8d61);
    assert_eq!(TestEnumGeneric::<1, u32>::CONST_HASH, 0x4dfc_9dc0_21f6_7ed3);
    assert_eq!(TestEnumGeneric::<2, u32>::CONST_HASH, 0x1d17_d768_1139_a46c);
    assert_eq!(
        TestEnumGeneric::<1, TestUnit>::CONST_HASH,
        0x49bd_7ea8_7af5_d8c7,
    );

    assert_eq!(TestUnion::CONST_HASH, 0x7a61_7d4b_e7ad_2011);
    assert_eq!(TestUnionGeneric::<u32>::CONST_HASH, 0x8c42_1a1f_ce70_ad8b);
    assert_eq!(TestUnionGeneric::<i64>::CONST_HASH, 0x3cdb_4a7b_0fd4_e42f);
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
