mod buffers;
mod combined_image_samplers;
mod constants;
mod decorations;
mod entry_points;
mod execution_modes;
mod names;
mod resources;
mod types;

use crate::{error, SpirvCrossError};
pub use buffers::*;
pub use combined_image_samplers::*;
pub use constants::*;
pub use decorations::*;
pub use entry_points::*;
pub use execution_modes::*;
pub use resources::*;
pub use types::*;

/// Check if an enum slice contains u32 max.
#[inline(always)]
fn enum_slice_is_not_max<T>(enum_slice: &[i32]) -> bool {
    #[cfg(target_endian = "big")]
    let memchr = memchr::memmem::find(
        bytemuck::must_cast_slice(enum_slice),
        &*i32::MAX.to_be_bytes(),
    );

    #[cfg(target_endian = "little")]
    let memchr = memchr::memmem::find(
        bytemuck::must_cast_slice(enum_slice),
        i32::MAX.to_le_bytes().as_slice(),
    );

    // memchr is only a heuristic, but if it doesn't find the bit pattern we're good.
    if memchr.is_none() {
        true
    } else {
        !enum_slice.contains(&i32::MAX)
    }
}

// SAFETY: ensure size is u32 first!
// won't need it after 1.79.
unsafe fn try_valid_slice<'a, T>(ptr: *const T, size: usize) -> error::Result<&'a [T]> {
    unsafe {
        let int_slice = std::slice::from_raw_parts(ptr.cast::<i32>(), size);
        if !enum_slice_is_not_max::<T>(int_slice) {
            Err(SpirvCrossError::InvalidEnum)
        } else {
            Ok(std::slice::from_raw_parts(ptr, size))
        }
    }
}
