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
use crate::string::MaybeCStr;
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

    pub fn set_name<'str, I: Id>(
        &mut self,
        handle: Handle<I>,
        string: impl Into<MaybeCStr<'str>>,
    ) -> error::Result<()> {
        let id = self.yield_id(handle)?;
        let string = string.into();

        unsafe {
            let Ok(cstring) = string.to_cstring_ptr() else {
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
    ) -> error::Result<Option<MaybeCStr<'a>>> {
        let struct_type_id = self.yield_id(struct_member.struct_type)?;
        let index = struct_member.index as u32;

        unsafe {
            let name = sys::spvc_compiler_get_member_name(self.ptr.as_ptr(), struct_type_id, index);
            let name = MaybeCStr::from_ptr(name);
            if name.is_empty() {
                Ok(None)
            } else {
                Ok(Some(name))
            }
        }
    }

    pub fn set_member_name<'str, I: Id>(
        &mut self,
        struct_member: StructMember<'a>,
        string: impl Into<MaybeCStr<'str>>,
    ) -> error::Result<()> {
        let struct_type_id = self.yield_id(struct_member.struct_type)?;
        let index = struct_member.index as u32;
        let string = string.into();

        unsafe {
            let Ok(cstring) = string.to_cstring_ptr() else {
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

    /// When declaring buffer blocks in GLSL, the name declared in the GLSL source
    /// might not be the same as the name declared in the SPIR-V module due to naming conflicts.
    /// In this case, SPIRV-Cross needs to find a fallback-name, and it might only
    /// be possible to know this name after compiling to GLSL.
    ///
    /// This is particularly important for HLSL input and UAVs which tends to reuse the same block type
    /// for multiple distinct blocks. For these cases it is not possible to modify the name of the type itself
    /// because it might be unique. Instead, you can use this interface to check after compilation which
    /// name was actually used if your input SPIR-V tends to have this problem.
    ///
    /// For other names like remapped names for variables, etc., it's generally enough to query the name of the variables
    /// after compiling, block names are an exception to this rule.
    /// ID is the name of a variable from [`Resource::id`], and must be a variable with a Block-like type.
    ///
    /// This also applies to HLSL cbuffers.
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
