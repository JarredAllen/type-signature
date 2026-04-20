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

/// All three enum variant shapes in one type, for shape-coverage hashing.
#[derive(TypeSignature)]
enum TestEnumAllVariantShapes {
    Unit,
    Tuple(u32, i32),
    Struct { a: u32, b: String },
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

#[derive(TypeSignature)]
#[type_signature(rename = "TestStruct")]
struct TestStructRenamed {
    a: u32,
    b: String,
}

// Both of these renamed to the same signature name so we can verify that adding a skipped
// field leaves the hash unchanged.
#[derive(TypeSignature)]
#[type_signature(rename = "SkipTarget")]
struct TestStructSkipBaseline {
    a: u32,
    b: String,
}

#[derive(TypeSignature)]
#[type_signature(rename = "SkipTarget")]
struct TestStructWithSkippedField {
    a: u32,
    b: String,
    #[type_signature(skip)]
    _cached: u64,
}

#[derive(TypeSignature)]
#[type_signature(rename = "SkipEnum")]
enum TestEnumSkipBaseline {
    A(u32, i32),
    B { a: u32, b: String },
}

#[derive(TypeSignature)]
#[type_signature(rename = "SkipEnum")]
enum TestEnumWithSkippedField {
    A(u32, i32),
    B {
        a: u32,
        b: String,
        #[type_signature(skip)]
        _cached: u64,
    },
}

// Field-level rename: the renamed field should hash to the same value as a field declared
// under the replacement name directly.
#[derive(TypeSignature)]
#[type_signature(rename = "FieldRenameTarget")]
struct TestFieldRenameBaseline {
    original: u32,
}

#[derive(TypeSignature)]
#[type_signature(rename = "FieldRenameTarget")]
struct TestFieldRenamed {
    #[type_signature(rename = "original")]
    renamed: u32,
}

// Lifetime parameters should not affect the signature.
#[derive(TypeSignature)]
#[type_signature(rename = "LifetimeTarget")]
struct TestLifetimeBaseline {
    x: u32,
}

#[derive(TypeSignature)]
#[type_signature(rename = "LifetimeTarget")]
struct TestLifetimeWith<'a> {
    x: u32,
    #[type_signature(skip)]
    _marker: core::marker::PhantomData<&'a ()>,
}

// Where-clause bounds should not affect the signature.
#[derive(TypeSignature)]
#[type_signature(rename = "WhereClauseTarget")]
struct TestWhereClauseBaseline<T> {
    x: T,
}

#[derive(TypeSignature)]
#[type_signature(rename = "WhereClauseTarget")]
struct TestWhereClauseWith<T>
where
    T: Clone,
{
    x: T,
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

    assert_eq!(TestEnumAllVariantShapes::CONST_HASH, 0x7c70_4007_50e1_0228,);
}

/// Golden hashes for stdlib types. These are part of the public API: any change to the
/// hashing algorithm, to the stdlib-impl wire format, or to rustc's `stringify!` output
/// (which currently feeds the `ty_name` for these impls) would change these values and
/// must be treated as a breaking change.
#[test]
fn test_stdlib_type_hashes() {
    assert_eq!(<u32 as TypeSignature>::CONST_HASH, 0xf8aa_761f_cd69_64d0);
    assert_eq!(<bool as TypeSignature>::CONST_HASH, 0x550b_d78c_c302_a37b);
    assert_eq!(<() as TypeSignature>::CONST_HASH, 0xfad5_5617_67b8_d0f2);
    assert_eq!(
        <(u8, u16) as TypeSignature>::CONST_HASH,
        0x3551_da08_87ec_9fa3,
    );
    assert_eq!(
        <Option<String> as TypeSignature>::CONST_HASH,
        0xe997_8466_7093_837f,
    );
    assert_eq!(
        <Vec<u32> as TypeSignature>::CONST_HASH,
        0x32f8_868e_a774_e1b7,
    );
    assert_eq!(
        <[u32; 4] as TypeSignature>::CONST_HASH,
        0x1986_3530_5e5a_13b9
    );
    assert_eq!(
        <[u32; 5] as TypeSignature>::CONST_HASH,
        0x0da7_82e3_6dee_5f22
    );
    assert_eq!(
        <Result<u32, String> as TypeSignature>::CONST_HASH,
        0x138f_694b_a8e1_8d1c,
    );
}

#[test]
fn test_lifetime_does_not_affect_signature() {
    // A lifetime-parameterized struct should hash identically to an otherwise-identical
    // struct without the lifetime.
    assert_eq!(
        TestLifetimeBaseline::CONST_HASH,
        TestLifetimeWith::<'static>::CONST_HASH,
    );
}

#[test]
fn test_where_clause_does_not_affect_signature() {
    // A where-clause bound (`T: Clone`) should not affect the signature.
    assert_eq!(
        TestWhereClauseBaseline::<u32>::CONST_HASH,
        TestWhereClauseWith::<u32>::CONST_HASH,
    );
}

#[test]
fn test_rename_attribute_matches_original() {
    // `#[type_signature(rename = "TestStruct")]` should produce the same signature as the
    // original type it renames to.
    assert_eq!(TestStruct::CONST_HASH, TestStructRenamed::CONST_HASH);
}

#[test]
fn test_field_rename_attribute_matches_original() {
    // Renaming a field to its previous name should preserve the signature.
    assert_eq!(
        TestFieldRenameBaseline::CONST_HASH,
        TestFieldRenamed::CONST_HASH,
    );
}

#[test]
fn test_skip_attribute_omits_field() {
    // A struct with a `#[type_signature(skip)]` field should hash identically to one without
    // the skipped field.
    assert_eq!(
        TestStructSkipBaseline::CONST_HASH,
        TestStructWithSkippedField::CONST_HASH,
    );
    // Skipping also works inside enum variants.
    assert_eq!(
        TestEnumSkipBaseline::CONST_HASH,
        TestEnumWithSkippedField::CONST_HASH,
    );
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
