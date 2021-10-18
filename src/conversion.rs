use bitvec::{order::Lsb0, ptr::BitPtr, slice::BitSlice};

pub fn byte_to_bits(byte: &u8) -> Option<&BitSlice<Lsb0, u8>> {
    let raw_bits = bitvec::ptr::bitslice_from_raw_parts::<Lsb0, u8>(BitPtr::from_ref(byte), 8);
    let bits;
    unsafe {
        bits = raw_bits.as_ref();
    }

    bits
}
