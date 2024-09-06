use crate::error;
use crate::handle::{Handle, Id};
use crate::Compiler;

use crate::string::CompilerStr;
use spirv_cross_sys as sys;
use spirv_cross_sys::{SpvId, TypeId, VariableId};

impl<T> Compiler<T> {
    /// Gets the identifier (`OpName`) of an ID.
    pub fn name<I: Id>(&self, handle: Handle<I>) -> error::Result<Option<CompilerStr>> {
        let id = self.yield_id(handle)?;
        unsafe {
            let name = sys::spvc_compiler_get_name(self.ptr.as_ptr(), SpvId(id.id()));
            let name = CompilerStr::from_ptr(name, self.ctx.drop_guard());
            if name.is_empty() {
                Ok(None)
            } else {
                Ok(Some(name))
            }
        }
    }

    /// Overrides the identifier OpName of an ID.
    ///
    /// Identifiers beginning with underscores or identifiers which contain double underscores
    /// are reserved by the implementation.
    pub fn set_name<'str, I: Id>(
        &mut self,
        handle: Handle<I>,
        string: impl Into<CompilerStr<'str>>,
    ) -> error::Result<()> {
        let id = self.yield_id(handle)?;
        let string = string.into();

        unsafe {
            let cstring = string.into_cstring_ptr()?;

            sys::spvc_compiler_set_name(self.ptr.as_ptr(), SpvId(id.id()), cstring.as_ptr());

            // Sanity drop to show that the lifetime of the cstring is only up until
            // we have returned. AFAIK, SPIRV-Cross will do a string copy.
            // If it does not, then we'll have to keep this string alive for a while.
            drop(cstring);
            Ok(())
        }
    }

    /// Given a struct type ID, obtain the identifier for member number "index".
    pub fn member_name(
        &self,
        struct_type: Handle<TypeId>,
        index: u32,
    ) -> error::Result<Option<CompilerStr>> {
        let struct_type_id = self.yield_id(struct_type)?;
        let index = index;

        unsafe {
            let name = sys::spvc_compiler_get_member_name(self.ptr.as_ptr(), struct_type_id, index);
            let name = CompilerStr::from_ptr(name, self.ctx.drop_guard());
            if name.is_empty() {
                Ok(None)
            } else {
                Ok(Some(name))
            }
        }
    }

    /// Sets the member identifier for the given struct member.
    pub fn set_member_name<'str>(
        &mut self,
        struct_type: Handle<TypeId>,
        index: u32,
        string: impl Into<CompilerStr<'str>>,
    ) -> error::Result<()> {
        let struct_type_id = self.yield_id(struct_type)?;
        let index = index;
        let string = string.into();

        unsafe {
            let cstring = string.into_cstring_ptr()?;

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
}

impl<T> Compiler<T> {
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
    /// `handle` should be a handle to a variable with a Block-like type.
    ///
    /// This also applies to HLSL cbuffers.
    pub fn remapped_declared_block_name(
        &self,
        handle: impl Into<Handle<VariableId>>,
    ) -> error::Result<Option<CompilerStr<'static>>> {
        let handle = handle.into();
        let handle = self.yield_id(handle)?;
        unsafe {
            let name =
                sys::spvc_compiler_get_remapped_declared_block_name(self.ptr.as_ptr(), handle);

            // SAFETY: 'ctx is sound here
            // https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross_c.cpp#L2773
            let name = CompilerStr::from_ptr(name, self.ctx.drop_guard());
            if name.is_empty() {
                Ok(None)
            } else {
                Ok(Some(name))
            }
        }
    }
}
