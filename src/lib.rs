//! This crate allows reading bytes representation of structs
//! even in presence of padding bytes.
//!
//! [![crates](https://img.shields.io/crates/v/safe-bytes.svg?label=safe-bytes)](https://crates.io/crates/safe-bytes)
//! [![docs](https://docs.rs/safe-bytes/badge.svg)](https://docs.rs/safe-bytes)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
//! [![License](https://img.shields.io/badge/license-APACHE-blue.svg)](LICENSE-APACHE)
//!
//! Simply derive [`SafeBytes`] for structures
//! where all field types are [`SafeBytes`] implementations.
//! And [`SafeBytes::safe_bytes`] would initialize all padding bytes
//! before returning `&[u8]`.
//! All primitives implement [`SafeBytes`] as there is no padding bytes.
//! Additionally some std types implement [`SafeBytes`].
//!
//! Note that in order to initialize padding bytes
//! [`SafeBytes::safe_bytes`] takes mutable reference `&mut self`.
//! And returns shareable reference `&[u8]` because not all
//! bitpatterns may be allowed for the type.
//!
//! [`SafeBytes`]: ./trait.SafeBytes.html
//! [`SafeBytes::safe_bytes`]: ./trait.SafeBytes.html#tymethod.safe_bytes

#![no_std]

mod pod;

use core::{
    mem::{size_of, size_of_val, ManuallyDrop, MaybeUninit},
    num::Wrapping,
    slice::{from_raw_parts, from_raw_parts_mut},
};

pub use safe_bytes_derive::SafeBytes;

#[doc(hidden)]
pub use core;

/// Creates [`TypeField`] for fieled of the given instance.
/// Can be used to implement [`PaddingBane::get_fields`].
///
/// [`TypeField`]: ./struct.TypedField.html
/// [`PaddingBane::get_fields`]: ./trait.PaddingBane.html#tymethod.get_fields
#[macro_export]
macro_rules! typed_field {
    ($instance:expr, $type:path, $field:ident) => {{
        let reference: &$type = &$instance;
        let $type {
            $field: field_reference,
            ..
        } = reference;
        let base_address = reference as *const _ as usize;
        let field_size = $crate::core::mem::size_of_val(field_reference);
        let field_address = field_reference as *const _ as usize;
        let field_offset = field_address.checked_sub(base_address).unwrap();
        let field_sub = $crate::PaddingBane::get_fields(field_reference);

        TypedField {
            raw: $crate::Field {
                offset: field_offset,
                size: field_size,
            },
            sub: field_sub,
        }
    }};
}

/// Trait for types that can initialize their padding in
/// their bytes representation.
///
/// # Safety
///
/// This trait's implementation *should* be able to produce initialized
/// bytes slice that contain original structure raw
/// representation.
/// It is safe to implement it in wrong way.
/// But it is advised to derive [`SafeBytes`] if possible.
/// Or implement [`PaddingBane`] instead if deriving is not possible,
/// as [`SafeBytes`] has blanket implementation for [`PaddingBane`] implementors.
///
/// [`SafeBytes`]: ./trait.SafeBytes.html
/// [`PaddingBane`]: ./trait.PaddingBane.html
pub trait SafeBytes {
    /// Returns bytes representation of the value,
    /// initializing all padding bytes
    fn safe_bytes(&mut self) -> &[u8];
}

/// This trait must be implemented in order to fill padding bytes of an object.
pub unsafe trait PaddingBane {
    /// Metadata about type's fields.
    type Fields: Copy;

    /// Return fields metadata.
    ///
    /// # Safety
    ///
    /// This function must return equal value for any instance of the `Self` type.
    /// It exists only because reference to instance is required to
    /// fetch field offsets.
    fn get_fields(&self) -> Self::Fields;

    /// Fills padding bytes in the bytes array.
    /// Padding bytes are bytes where no fields of the struct are stored
    /// or padding bytes of the fields.
    ///
    /// # Safety
    ///
    /// `fields` must be created from any instance of `Self`.
    /// `bytes` must be created by casting `&mut Self` or, for a field,
    /// it must be subslice of the parent's bytes where field is stored.
    unsafe fn init_padding(fields: Self::Fields, bytes: &mut [MaybeUninit<u8>]);
}

impl<T> SafeBytes for T
where
    T: PaddingBane,
{
    #[inline]
    fn safe_bytes(&mut self) -> &[u8] {
        let fields = self.get_fields();
        unsafe {
            let bytes = maybe_init_bytes_of(self);
            Self::init_padding(fields, bytes);
            assume_slice_init(&*bytes)
        }
    }
}

impl<T> SafeBytes for [T]
where
    T: PaddingBane,
{
    fn safe_bytes(&mut self) -> &[u8] {
        if self.is_empty() {
            &[]
        } else {
            let fields = self[0].get_fields();
            let len = self.len();
            unsafe {
                let bytes = maybe_init_bytes_of(self);
                for i in 0..len {
                    let start = i * size_of::<T>();
                    let end = start + size_of::<T>();
                    T::init_padding(fields, &mut bytes[start..end]);
                }
                assume_slice_init(&*bytes)
            }
        }
    }
}

macro_rules! impl_for_array {
    ($N:tt) => {
        unsafe impl<T> PaddingBane for [T; $N]
        where
            T: PaddingBane,
        {
            type Fields = T::Fields;
            #[inline(always)]
            fn get_fields(&self) -> T::Fields {
                self[0].get_fields()
            }

            #[inline(always)]
            unsafe fn init_padding(fields: T::Fields, bytes: &mut [MaybeUninit<u8>]) {
                for i in 0 .. $N {
                    let start = i * size_of::<T>();
                    let end = start + size_of::<T>();
                    T::init_padding(fields, &mut bytes[start..end]);
                }
            }
        }
    };

    ($($N:tt)*) => {
        $(impl_for_array!($N);)*
    };
}

impl_for_array! {
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
    48 64 96 128 256 512 1024 2048 4096 8192 16384 32768 65536
}

unsafe impl<T> PaddingBane for ManuallyDrop<T>
where
    T: PaddingBane,
{
    type Fields = T::Fields;

    #[inline(always)]
    fn get_fields(&self) -> Self::Fields {
        (&**self).get_fields()
    }

    #[inline(always)]
    unsafe fn init_padding(fields: Self::Fields, bytes: &mut [MaybeUninit<u8>]) {
        T::init_padding(fields, bytes);
    }
}

unsafe impl<T> PaddingBane for Wrapping<T>
where
    T: PaddingBane,
{
    type Fields = T::Fields;

    #[inline(always)]
    fn get_fields(&self) -> Self::Fields {
        self.0.get_fields()
    }

    #[inline(always)]
    unsafe fn init_padding(fields: Self::Fields, bytes: &mut [MaybeUninit<u8>]) {
        T::init_padding(fields, bytes);
    }
}

/// Basic field information.
/// Enough to fill padding bytes between fields.
#[derive(Clone, Copy)]
pub struct Field {
    pub offset: usize,
    pub size: usize,
}

/// Field information.
/// Enough to fill padding bytes between fields and
/// inside the fields.
#[derive(Clone, Copy)]
pub struct TypedField<T: PaddingBane> {
    pub raw: Field,
    pub sub: T::Fields,
}

/// Returns maybe uninitialized bytes of the value.
/// Intended for initializing padding bytes.
///
/// # Safety
///
/// Returned bytes reference must not be used to create invalid bit pattern.
unsafe fn maybe_init_bytes_of<T: ?Sized>(r: &mut T) -> &mut [MaybeUninit<u8>] {
    from_raw_parts_mut(r as *mut T as *mut MaybeUninit<u8>, size_of_val(r))
}

/// Assume all elements of the slice are initialized.
///
/// # Safety
///
/// All elements of the slice must be initialized.
unsafe fn assume_slice_init<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    from_raw_parts(slice.as_ptr() as *const T, size_of_val(slice))
}
