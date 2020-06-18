use {
    core::{
        mem::{size_of, MaybeUninit},
        ptr::write_bytes,
    },
    safe_bytes::{typed_field, PaddingBane, SafeBytes, TypedField},
};

/// Example custom implementation for struct with padding bytes
#[repr(C)]
pub struct Example {
    pub a: u8,
    pub b: u64,
    pub c: u16,
}

#[derive(Clone, Copy)]
pub struct ExamplesFields {
    pub a_field: TypedField<u8>,
    pub b_field: TypedField<u64>,
    pub c_field: TypedField<u16>,
}

unsafe impl PaddingBane for Example {
    type Fields = ExamplesFields;

    fn get_fields(&self) -> Self::Fields {
        let a_field = typed_field!(*self, Example, a);
        let b_field = typed_field!(*self, Example, b);
        let c_field = typed_field!(*self, Example, c);

        ExamplesFields {
            a_field,
            b_field,
            c_field,
        }
    }

    unsafe fn init_padding(fields: ExamplesFields, bytes: &mut [MaybeUninit<u8>]) {
        let ExamplesFields {
            a_field,
            b_field,
            c_field,
        } = fields;

        // First find offsets and sizes of all fields.
        let mut fields = [a_field.raw, b_field.raw, c_field.raw];

        // Sort fields by offset.
        fields.sort_unstable_by_key(|f| f.offset);

        // Find any padding between fields and fill it.
        let mut offset = 0;
        for field in &fields {
            if field.offset > offset {
                let count = field.offset - offset;
                // Fill padding.
                write_bytes(&mut bytes[offset], 0xfe, count);
            }
            offset = field.offset + field.size;
        }

        // Padding at the end
        if size_of::<Self>() > offset {
            let count = size_of::<Self>() - offset;
            write_bytes(&mut bytes[offset], 0xfe, count);
        }

        // Repeat recursively for each field.
        let a_bytes = &mut bytes[a_field.raw.offset..a_field.raw.offset + a_field.raw.size];
        <u8 as PaddingBane>::init_padding(a_field.sub, a_bytes);

        let b_bytes = &mut bytes[b_field.raw.offset..b_field.raw.offset + b_field.raw.size];
        <u64 as PaddingBane>::init_padding(b_field.sub, b_bytes);

        let c_bytes = &mut bytes[c_field.raw.offset..c_field.raw.offset + a_field.raw.size];
        <u16 as PaddingBane>::init_padding(c_field.sub, c_bytes);
    }
}

#[derive(SafeBytes)]
#[repr(C)]
pub struct Example2 {
    a: u8,
    b: u64,
    c: u16,
}

#[cfg(target_endian = "big")]
const SAFE_BYTES: [u8; 24] = [
    0x01, // a
    0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, // pad
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, // b,
    0x00, 0x00, // c,
    0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, // pad
];

#[cfg(target_endian = "little")]
const SAFE_BYTES: [u8; 24] = [
    0x01, // a
    0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, // pad
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // b,
    0x03, 0x00, // c,
    0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, // pad
];

fn main() {
    let mut example = Example { a: 1, b: 2, c: 3 };
    let bytes = example.safe_bytes();
    assert_eq!(bytes, &SAFE_BYTES);

    let mut example = Example2 { a: 1, b: 2, c: 3 };
    let bytes = example.safe_bytes();
    assert_eq!(bytes, &SAFE_BYTES);
}
