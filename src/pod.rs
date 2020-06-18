use {
    crate::PaddingBane,
    core::{
        marker::{PhantomData, PhantomPinned},
        mem::MaybeUninit,
        num::{
            NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
            NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
        },
        ptr::NonNull,
        sync::atomic::{
            AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
            AtomicU64, AtomicU8, AtomicUsize,
        },
    },
};

macro_rules! impl_pod {
    ($(<$($g:ident $(:$b:path)?),+>)? for $t:ty) => {
        unsafe impl $(<$($g $(:$b)?),+>)? PaddingBane for $t {
            type Fields = PhantomData<fn($t) -> $t>;

            #[inline(always)]
            fn get_fields(&self) -> Self::Fields {
                PhantomData
            }

            #[inline(always)]
            unsafe fn init_padding(_fields: Self::Fields, _bytes: &mut [MaybeUninit<u8>]) {}
        }
    };
}

impl_pod!(for ());
impl_pod!(for bool);
impl_pod!(for u8);
impl_pod!(for i8);
impl_pod!(for u16);
impl_pod!(for i16);
impl_pod!(for u32);
impl_pod!(for i32);
impl_pod!(for u64);
impl_pod!(for i64);
impl_pod!(for usize);
impl_pod!(for isize);
impl_pod!(for u128);
impl_pod!(for i128);
impl_pod!(for f32);
impl_pod!(for f64);

impl_pod!(for AtomicU8);
impl_pod!(for AtomicI8);
impl_pod!(for AtomicU16);
impl_pod!(for AtomicI16);
impl_pod!(for AtomicU32);
impl_pod!(for AtomicI32);
impl_pod!(for AtomicU64);
impl_pod!(for AtomicI64);
impl_pod!(for AtomicUsize);
impl_pod!(for AtomicIsize);

impl_pod!(for Option<NonZeroI8>);
impl_pod!(for Option<NonZeroI16>);
impl_pod!(for Option<NonZeroI32>);
impl_pod!(for Option<NonZeroI64>);
impl_pod!(for Option<NonZeroI128>);
impl_pod!(for Option<NonZeroIsize>);
impl_pod!(for Option<NonZeroU8>);
impl_pod!(for Option<NonZeroU16>);
impl_pod!(for Option<NonZeroU32>);
impl_pod!(for Option<NonZeroU64>);
impl_pod!(for Option<NonZeroU128>);
impl_pod!(for Option<NonZeroUsize>);

impl_pod!(<T> for *mut T);
impl_pod!(<T> for *const T);
impl_pod!(<T> for Option<NonNull<T>>);
impl_pod!(<T> for PhantomData<T>);
impl_pod!(for PhantomPinned);
impl_pod!(<T> for [T; 0]);
