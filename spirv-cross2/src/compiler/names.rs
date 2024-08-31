// TODO:

// const char *argument);
// SPVC_PUBLIC_API void spvc_compiler_set_name(spvc_compiler compiler, SpvId id, const char *argument);
//
// const char *argument);
// SPVC_PUBLIC_API void spvc_compiler_set_member_name(spvc_compiler compiler, spvc_type_id id, unsigned member_index,
// const char *argument);

//

// SPVC_PUBLIC_API const char *spvc_compiler_get_name(spvc_compiler compiler, SpvId id);
// SPVC_PUBLIC_API const char *spvc_compiler_get_member_name(spvc_compiler compiler, SpvId id);

use crate::compiler::Compiler;
use crate::error;
use crate::handle::{Handle, Id};
use std::borrow::Cow;
use std::ffi::{CStr, CString};

use crate::compiler::types::StructMember;
use crate::error::SpirvCrossError;
use spirv_cross_sys as sys;
use spirv_cross_sys::{SpvId, VariableId};

impl<'a, T> Compiler<'a, T> {
    pub fn name<I: Id>(&self, handle: Handle<I>) -> error::Result<Option<Cow<'a, str>>> {
        let id = self.yield_id(handle)?;
        unsafe {
            let name = sys::spvc_compiler_get_name(self.ptr.as_ptr(), SpvId(id.id()));
            let name = CStr::from_ptr(name);
            if name.is_empty() {
                Ok(None)
            } else {
                Ok(Some(name.to_string_lossy()))
            }
        }
    }

    pub fn set_name<I: Id>(&self, handle: Handle<I>, string: impl AsRef<str>) -> error::Result<()> {
        let id = self.yield_id(handle)?;

        unsafe {
            let Ok(cstring) = CString::new(String::from(string.as_ref())) else {
                return Err(SpirvCrossError::InvalidName(String::from(string.as_ref())));
            };

            sys::spvc_compiler_set_name(self.ptr.as_ptr(), SpvId(id.id()), cstring.as_ptr());

            // Sanity drop to show that the lifetime of the cstring is only up until
            // we have returned. AFAIK, SPIRV-Cross will do a string copy.
            // If it does not, then we'll have to keep this string alive for a while.
            drop(cstring);
            Ok(())
        }
    }

    pub fn member_name<I: Id>(
        &self,
        struct_member: StructMember<'a>,
    ) -> error::Result<Option<Cow<'a, str>>> {
        let struct_type_id = self.yield_id(struct_member.struct_type)?;
        let index = struct_member.index as u32;

        unsafe {
            let name = sys::spvc_compiler_get_member_name(self.ptr.as_ptr(), struct_type_id, index);
            let name = CStr::from_ptr(name);
            if name.is_empty() {
                Ok(None)
            } else {
                Ok(Some(name.to_string_lossy()))
            }
        }
    }

    pub fn set_member_name<I: Id>(
        &self,
        struct_member: StructMember<'a>,
        string: impl AsRef<str>,
    ) -> error::Result<()> {
        let struct_type_id = self.yield_id(struct_member.struct_type)?;
        let index = struct_member.index as u32;

        unsafe {
            let Ok(cstring) = CString::new(String::from(string.as_ref())) else {
                return Err(SpirvCrossError::InvalidName(String::from(string.as_ref())));
            };

            sys::spvc_compiler_set_member_name(
                self.ptr.as_ptr(),
                struct_type_id,
                index,
                cstring.as_ptr(),
            );

            // Sanity drop to show that the lifetime of the cstring is only up until
            // we have returned. AFAIK, SPIRV-Cross will do a string copy.
            // If it does not, then we'll have to keep this string alive for a while.
            drop(cstring);
            Ok(())
        }
    }

    pub fn remapped_declared_block_name(
        &self,
        handle: Handle<VariableId>,
    ) -> error::Result<Option<Cow<'a, str>>> {
        let handle = self.yield_id(handle)?;
        unsafe {
            let name =
                sys::spvc_compiler_get_remapped_declared_block_name(self.ptr.as_ptr(), handle);
            let name = CStr::from_ptr(name);
            if name.is_empty() {
                Ok(None)
            } else {
                Ok(Some(name.to_string_lossy()))
            }
        }
    }
}
