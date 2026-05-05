#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// A type that can be made into a signature.
///
/// If implementing for a custom type, prefer to use the derive macro. Note that the derive macro
/// requires all fields to implement this trait, as well as all generic arguments.
///
/// # What affects the signature
///
/// The signature captures the structural shape of the type. The following changes rotate
/// the hash:
///
/// - Renaming the type, a field, or an enum variant (unless `#[type_signature(rename = "...")]`
///   is applied to keep the signature's view of the name unchanged).
/// - Adding, removing, or reordering fields or variants.
/// - Changing a field's type.
/// - Adding, removing, or reordering generic type parameters.
/// - Changing the value of a const generic.
/// - Converting between a tuple struct and a named-field struct (or equivalent changes to
///   enum variant shapes).
///
/// The following changes leave the hash untouched:
///
/// - Field or type visibility (`pub` vs private).
/// - Trait and method implementations on the type.
/// - Doc comments and attributes other than `#[type_signature(...)]` (including `#[repr(..)]`
///   attributes!).
/// - Adding, removing, or modifying fields marked `#[type_signature(skip)]`.
/// - Lifetime parameters and where-clause bounds.
///
/// # Derive attributes
///
/// The derive macro accepts `#[type_signature(...)]` attributes to customize what goes into
/// the signature:
///
/// - `#[type_signature(rename = "...")]` on the type — use the given name in the signature
///   instead of the type's own identifier. Useful for keeping a signature stable across a type
///   rename, or for matching the signature of a type in another crate.
/// - `#[type_signature(crate = "...")]` on the type — use the given path to refer to this crate
///   (instead of the default `type-signature`), which can be useful if calling the derive macro
///   from another crate re-exporting the trait.
/// - `#[type_signature(rename = "...")]` on an enum variant — use the given name for this
///   variant in the signature instead of the variant's own identifier. Useful for renaming a
///   variant without breaking the signature.
/// - `#[type_signature(rename = "...")]` on a field — use the given name for this field in
///   the signature instead of the field's own identifier. Useful for renaming a field without
///   breaking the signature.
/// - `#[type_signature(skip)]` on a field — exclude the field from the signature. Use this for
///   implementation-detail fields (caches, telemetry counters, `PhantomData`) whose presence
///   shouldn't count as a breaking change to the type's observable contract.
///
/// ```
/// use type_signature::TypeSignature;
///
/// // `rename` preserves the signature across a type rename.
/// #[derive(TypeSignature)]
/// struct Original {
///     x: u32,
/// }
///
/// #[derive(TypeSignature)]
/// #[type_signature(rename = "Original")]
/// struct Renamed {
///     x: u32,
/// }
///
/// assert_eq!(Original::CONST_HASH, Renamed::CONST_HASH);
///
/// // Field-level `rename` preserves the signature when a field is renamed.
/// #[derive(TypeSignature)]
/// struct HasFoo { foo: u32 }
///
/// #[derive(TypeSignature)]
/// #[type_signature(rename = "HasFoo")]
/// struct HasBar {
///     #[type_signature(rename = "foo")]
///     bar: u32,
/// }
///
/// assert_eq!(HasFoo::CONST_HASH, HasBar::CONST_HASH);
///
/// // `skip` lets implementation-detail fields be added without changing the signature.
/// // (Combined with `rename` here to simulate an in-place evolution of the same type.)
/// #[derive(TypeSignature)]
/// struct V1 {
///     id: u32,
/// }
///
/// #[derive(TypeSignature)]
/// #[type_signature(rename = "V1")]
/// struct V2 {
///     id: u32,
///     #[type_signature(skip)]
///     cached: u64,
/// }
///
/// assert_eq!(V1::CONST_HASH, V2::CONST_HASH);
/// ```
pub trait TypeSignature {
    /// The signature of this type.
    ///
    /// For a fixed type, if the hash of this value as produced by the derive macro changes, it
    /// will be treated as a breaking change.
    const SIGNATURE: TypeSignatureHasher;

    /// A const-available u64 value.
    ///
    /// For a fixed type, if this value as produced by the derive macro changes, it will be treated
    /// as a breaking change.
    const CONST_HASH: u64 = Self::SIGNATURE.const_hash();
}

pub use type_signature_derive::TypeSignature;

/// A hashable type for generating a signature for a type.
///
/// The fields of this struct are not considered a stable API contract, and are not to be
/// explicitly referenced in calling code.
#[derive(Debug, Hash)]
pub struct TypeSignatureHasher {
    /// The name of the type being hashed.
    #[doc(hidden)]
    pub ty_name: &'static str,
    /// The types of the generic arguments.
    #[doc(hidden)]
    pub ty_generics: &'static [&'static TypeSignatureHasher],
    /// Hashes of const generics.
    #[doc(hidden)]
    pub const_generic_hashes: &'static [u64],
    /// The "variants" of a type.
    ///
    /// For each "variant", it has the name of the variant and a list of the field names and types.
    #[doc(hidden)]
    pub variants: &'static [(
        &'static str,
        &'static [(&'static str, &'static TypeSignatureHasher)],
    )],
}
impl TypeSignatureHasher {
    /// Generate a hash at compile time.
    ///
    /// This function exists to cover for the inability to call [`core::hash::Hash::hash`] at const-time, and
    /// will likely be deprecated once const traits exist.
    #[must_use]
    pub const fn const_hash(&self) -> u64 {
        let mut accumulator = 0x1b61_42dc_8803_64ed;

        // Mix in the name of the type
        __macro_export::mix_values(&mut accumulator, __macro_export::hash_str(self.ty_name));

        // Mix in the types of each generic. Length first so boundaries are unambiguous.
        {
            __macro_export::mix_values(&mut accumulator, self.ty_generics.len() as u64);
            let mut generic_idx = 0;
            while generic_idx < self.ty_generics.len() {
                __macro_export::mix_values(
                    &mut accumulator,
                    self.ty_generics[generic_idx].const_hash(),
                );
                generic_idx += 1;
            }
        }

        // Mix in each const generic argument
        {
            __macro_export::mix_values(&mut accumulator, self.const_generic_hashes.len() as u64);
            let mut const_generic_idx = 0;
            while const_generic_idx < self.const_generic_hashes.len() {
                __macro_export::mix_values(
                    &mut accumulator,
                    self.const_generic_hashes[const_generic_idx],
                );
                const_generic_idx += 1;
            }
        }

        // Mix in the types and names of each field.
        {
            __macro_export::mix_values(&mut accumulator, self.variants.len() as u64);
            let mut variant_idx = 0;
            while variant_idx < self.variants.len() {
                let (variant_name, variant_fields) = self.variants[variant_idx];
                __macro_export::mix_values(
                    &mut accumulator,
                    __macro_export::hash_str(variant_name),
                );
                __macro_export::mix_values(&mut accumulator, variant_fields.len() as u64);
                let mut field_idx = 0;
                while field_idx < variant_fields.len() {
                    let (field_name, field_hasher) = variant_fields[field_idx];
                    __macro_export::mix_values(
                        &mut accumulator,
                        __macro_export::hash_str(field_name),
                    );
                    __macro_export::mix_values(&mut accumulator, field_hasher.const_hash());
                    field_idx += 1;
                }
                variant_idx += 1;
            }
        }

        accumulator
    }
}

/// Provide an implementation for a stdlib type.
macro_rules! impl_for_stdlib_ty {
    ($(
        $stdty:ty $(where < $( $generic:ident $( : $generic_cond:tt )? ),* > )?
    ),+ $(,)?) => {$(
        impl$( < $( $generic $( : $generic_cond )? ),* > )? $crate::TypeSignature for $stdty {
            const SIGNATURE: $crate::TypeSignatureHasher  = $crate::TypeSignatureHasher {
                // `ty_name` in derive macro is just the type name, so this should avoid possible
                // conflicts.
                ty_name: concat!(stringify!($crate::TypeSignature), " impl for ", stringify!($stdty)),
                ty_generics: &[
                    $( $( &<$generic as $crate::TypeSignature>::SIGNATURE, )* )?
                ],
                const_generic_hashes: &[],
                // Not formally correct, but good enough for stdlib types since they won't change
                variants: &[],
            };
        }
    )+};
}

impl_for_stdlib_ty!(
    u8, u16, u32, usize, u64, u128,
    i8, i16, i32, isize, i64, i128,
    bool,
    f32,
    f64,
    char,
    str,
    (),
    &T where <T: TypeSignature>,
    &mut T where <T: TypeSignature>,
    *const T where <T: TypeSignature>,
    *mut T where <T: TypeSignature>,
    [T] where <T: TypeSignature>,
    Option<T> where <T: TypeSignature> ,
    Result<T, E> where <T: TypeSignature, E: TypeSignature>,
    core::marker::PhantomData<T> where <T: TypeSignature>,
    core::mem::MaybeUninit<T> where <T: TypeSignature>,
    core::mem::ManuallyDrop<T> where <T: TypeSignature>,
    core::net::IpAddr, core::net::Ipv4Addr, core::net::Ipv6Addr,
    core::net::SocketAddr, core::net::SocketAddrV4, core::net::SocketAddrV6,
    core::num::NonZeroU8, core::num::NonZeroU16, core::num::NonZeroU32, core::num::NonZeroU64, core::num::NonZeroUsize, core::num::NonZeroU128,
    core::num::NonZeroI8, core::num::NonZeroI16, core::num::NonZeroI32, core::num::NonZeroI64, core::num::NonZeroIsize, core::num::NonZeroI128,
    core::num::Saturating<T> where <T: TypeSignature>,
    core::num::Wrapping<T> where <T: TypeSignature>,
    core::ops::Range<T> where <T: TypeSignature>,
    core::ops::RangeFrom<T> where <T: TypeSignature>,
    core::ops::RangeFull,
    core::ops::RangeInclusive<T> where <T: TypeSignature>,
    core::ops::RangeTo<T> where <T: TypeSignature>,
    core::ops::RangeToInclusive<T> where <T: TypeSignature>,
    core::pin::Pin<T> where <T: TypeSignature>,
    core::ptr::NonNull<T> where <T: TypeSignature>,
    core::cmp::Ordering,
    core::convert::Infallible,
    core::time::Duration,
);

#[cfg(target_has_atomic = "8")]
impl_for_stdlib_ty!(
    core::sync::atomic::AtomicBool,
    core::sync::atomic::AtomicI8,
    core::sync::atomic::AtomicU8,
);
#[cfg(target_has_atomic = "16")]
impl_for_stdlib_ty!(core::sync::atomic::AtomicI16, core::sync::atomic::AtomicU16);
#[cfg(target_has_atomic = "32")]
impl_for_stdlib_ty!(core::sync::atomic::AtomicI32, core::sync::atomic::AtomicU32);
#[cfg(target_has_atomic = "64")]
impl_for_stdlib_ty!(core::sync::atomic::AtomicI64, core::sync::atomic::AtomicU64);
#[cfg(target_has_atomic = "ptr")]
impl_for_stdlib_ty!(
    core::sync::atomic::AtomicIsize,
    core::sync::atomic::AtomicUsize,
    core::sync::atomic::AtomicPtr<T> where <T: TypeSignature>,
);

impl<const N: usize, T: TypeSignature> TypeSignature for [T; N] {
    const SIGNATURE: TypeSignatureHasher = TypeSignatureHasher {
        ty_name: "TypeSignature impl for [T; N]",
        ty_generics: &[&T::SIGNATURE],
        const_generic_hashes: &[__macro_export::hash_const_usize(N)],
        // Not formally correct, but good enough for stdlib types since they won't change
        variants: &[],
    };
}

/// Implement for a tuple of values which all implement [`TypeSignature`].
macro_rules! impl_for_tuple {
    ($(
        ( $( $elem_ty:ident, )* $(,)? )
    ),+ $(,)? ) => {$(

        impl< $( $elem_ty ),* > TypeSignature for ($($elem_ty,)*)
            where $( $elem_ty : TypeSignature ),*
        {
            const SIGNATURE: TypeSignatureHasher  = TypeSignatureHasher {
                ty_name: concat!(stringify!($crate::TypeSignature), " impl for (", $( stringify!($elem_ty), "," ),* ),
                ty_generics: &[
                    $( &<$elem_ty as $crate::TypeSignature>::SIGNATURE, )*
                ],
                const_generic_hashes: &[],
                // Not formally correct, but good enough for stdlib types since they won't change
                variants: &[],
            };
        }

    )+};
}

impl_for_tuple!(
    (T0,),
    (T0, T1,),
    (T0, T1, T2,),
    (T0, T1, T2, T3,),
    (T0, T1, T2, T3, T4,),
    (T0, T1, T2, T3, T4, T5,),
    (T0, T1, T2, T3, T4, T5, T6,),
    (T0, T1, T2, T3, T4, T5, T6, T7,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,),
    (
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14,
    ),
    (
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,
    ),
);

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
mod std_impl {
    extern crate std;

    use crate::TypeSignature;

    impl_for_stdlib_ty!(
        std::collections::HashMap<K, V> where <K: TypeSignature, V: TypeSignature>,
        std::collections::HashSet<T> where <T: TypeSignature>,
        std::ffi::OsStr,
        std::ffi::OsString,
        std::path::Path,
        std::path::PathBuf,
        std::sync::Mutex<T> where <T: TypeSignature>,
        std::sync::RwLock<T> where <T: TypeSignature>,
        std::sync::Once,
        std::time::Instant,
        std::time::SystemTime,
    );
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
mod alloc_impl {
    extern crate alloc;

    use crate::TypeSignature;

    impl_for_stdlib_ty!(
        alloc::boxed::Box<T> where <T: TypeSignature>,
        alloc::collections::BinaryHeap<T> where <T: TypeSignature>,
        alloc::collections::BTreeMap<K, V> where <K: TypeSignature, V: TypeSignature>,
        alloc::collections::BTreeSet<T> where <T: TypeSignature>,
        alloc::collections::LinkedList<T> where <T: TypeSignature>,
        alloc::collections::VecDeque<T> where <T: TypeSignature>,
        alloc::ffi::CString,
        alloc::rc::Rc<T> where <T: TypeSignature>,
        alloc::rc::Weak<T> where <T: TypeSignature>,
        alloc::string::String,
        alloc::vec::Vec<T> where <T: TypeSignature>,
    );

    #[cfg(target_has_atomic = "ptr")]
    impl_for_stdlib_ty!(
        alloc::sync::Arc<T> where <T: TypeSignature>,
        alloc::sync::Weak<T> where <T: TypeSignature>,
    );

    impl<'a, B: TypeSignature + alloc::borrow::ToOwned + ?Sized + 'a> TypeSignature
        for alloc::borrow::Cow<'a, B>
    {
        const SIGNATURE: crate::TypeSignatureHasher = crate::TypeSignatureHasher {
            ty_name: "TypeSignature impl for Cow<'a, B>",
            ty_generics: &[&<B as TypeSignature>::SIGNATURE],
            const_generic_hashes: &[],
            variants: &[],
        };
    }
}

/// Implement [`TypeSignature`] for the given type as though it had the given definition.
///
/// You probably just want to use the derive macro, which is able to do most simple transformations
/// by itself, but this macro may be useful in niche circumstances where the compiler-understood
/// fields don't match the underlying shape of the type (for example, if you're bit-packing
/// multiple flags into a single integer value).
///
/// Due to macro limitations, this macro requires the target type to have its name in scope at the
/// call site, and the macro can't expand paths.
///
///
/// # Example Use
/// ```
/// mod mod1 {
///     use type_signature::impl_type_signature_as;
///
///     pub struct Foo;
///     impl_type_signature_as! {
///         Foo as struct { a: u32 }
///     }
/// }
///
/// mod mod2 {
///     use type_signature::TypeSignature;
///
///     #[derive(TypeSignature)]
///     pub struct Foo { a: u32 }
/// }
///
/// use type_signature::TypeSignature;
///
/// assert_eq!(mod1::Foo::CONST_HASH, mod2::Foo::CONST_HASH);
/// ```
/// Or, the equivalent for enums:
/// ```
/// mod mod1 {
///     use type_signature::impl_type_signature_as;
///
///     pub struct Foo;
///     impl_type_signature_as! {
///         Foo as enum { A, B(u32) }
///     }
/// }
///
/// mod mod2 {
///     use type_signature::TypeSignature;
///
///     #[derive(TypeSignature)]
///     pub enum Foo { A, B(u32) }
/// }
///
/// use type_signature::TypeSignature;
///
/// assert_eq!(mod1::Foo::CONST_HASH, mod2::Foo::CONST_HASH);
/// ```
#[macro_export]
macro_rules! impl_type_signature_as {
    ($target:ident as struct { $( $fields:tt )* }) => {
        $crate::_impl_ts_as_helper! { $target as $target in struct $target { $( $fields )* } }
    };
    ($target:ident as enum { $( $variants:tt )* }) => {
        $crate::_impl_ts_as_helper! { $target as $target in enum $target { $( $variants )* } }
    };
}

/// Items exported only for use in the derive macro.
///
/// Do not treat anything in here like a public API.
#[doc(hidden)]
pub mod __macro_export {
    /// Hash a const `usize` value.
    #[must_use]
    pub const fn hash_const_usize(param_val: usize) -> u64 {
        let mut accumulator = hash_str("usize");
        mix_values(&mut accumulator, param_val as u64);
        // Handle 128-bit targets.
        if size_of::<usize>() == 16 {
            mix_values(&mut accumulator, (param_val >> 8) as u64);
        }
        accumulator
    }

    /// Hash a const `usize` value.
    #[must_use]
    pub const fn hash_const_bool(param_val: bool) -> u64 {
        let mut accumulator = hash_str("bool");
        mix_values(
            &mut accumulator,
            // Values chosen randomly to maximize number of bits different from any common pattern.
            if param_val {
                0x7907_e475_126f_2049
            } else {
                0xa656_face_e66f_d217
            },
        );
        accumulator
    }

    /// Mix a `u64` in to the accumulator.
    ///
    /// The mixing is done to ensure that the value is highly likely to change, and will likely
    /// be different for applying values in a different order.
    pub const fn mix_values(accumulator: &mut u64, value: u64) {
        // Constants are all primes, so multiplying and adding shuffles the values around
        // isomorphically.
        *accumulator = accumulator
            .wrapping_mul(0x35ce_5fac_9b48_99b5)
            .wrapping_add(0x1e5d_49b9_70ea_d075)
            ^ value
                .wrapping_mul(0x13fd_608d_551c_c1d1)
                .wrapping_add(0x87b5_2407_45ca_ca0f);
    }

    /// Hash a string into a fixed `u64`.
    ///
    /// This function is designed to quickly jumble the contents, and result in vastly different
    /// hashes for even subtly-different strings.
    #[must_use]
    pub const fn hash_str(s: &str) -> u64 {
        let mut accumulator = 0x1124_262e_5999_d5bb;
        mix_values(&mut accumulator, s.len() as u64);
        let mut byte_idx = 0;
        while byte_idx < s.len() {
            mix_values(&mut accumulator, s.as_bytes()[byte_idx] as u64);
            byte_idx += 1;
        }
        accumulator
    }

    /// Helper macro for [`crate::impl_type_signature_as`].
    #[doc(hidden)]
    #[macro_export]
    macro_rules! _impl_ts_as_helper {
        ($target:ty as $ident:ident in $item:item) => {
            impl $crate::TypeSignature for $target {
                const SIGNATURE: $crate::TypeSignatureHasher = {
                    #[derive($crate::TypeSignature)]
                    #[type_signature(crate = $crate)]
                    #[allow(dead_code)]
                    $item

                    $ident::SIGNATURE
                };
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(TypeSignature)]
    #[type_signature(crate = super)]
    #[allow(dead_code, reason = "Schema depends on it")]
    struct Foo {
        bar: u32,
    }

    #[test]
    fn test_derive_with_custom_crate_name() {
        assert_eq!(Foo::CONST_HASH, 0x9cb2_d1de_e1dc_8ae3);
    }
}
