use crate::cell::AllocationDropGuard;
use crate::error;
use crate::error::{SpirvCrossError, ToContextError};
use crate::handle::Handle;
use crate::iter::impl_iterator;
use crate::reflect::try_valid_slice;
use crate::string::CompilerStr;
use crate::Compiler;
use core::slice;
use spirv_cross_sys as sys;
use spirv_cross_sys::{spvc_entry_point, SpvBuiltIn, SpvExecutionModel, SpvStorageClass};
use std::ffi::c_char;

/// Iterator for declared extensions, created by [`Compiler::declared_extensions`].
pub struct ExtensionsIter<'a>(slice::Iter<'a, *const c_char>, AllocationDropGuard);

impl_iterator!(ExtensionsIter<'c>: CompilerStr<'c> as map |s, ptr: &*const c_char| {
    unsafe {
        CompilerStr::from_ptr(*ptr, s.1.clone())
    }
} for <'c> [0]);

/// Querying declared properties of the SPIR-V module.
impl<T> Compiler<T> {
    /// Gets the list of all SPIR-V Capabilities which were declared in the SPIR-V module.
    pub fn declared_capabilities(&self) -> error::Result<&[spirv::Capability]> {
        unsafe {
            let mut caps = std::ptr::null();
            let mut size = 0;

            sys::spvc_compiler_get_declared_capabilities(self.ptr.as_ptr(), &mut caps, &mut size)
                .ok(self)?;

            const _: () =
                assert!(std::mem::size_of::<spirv::Capability>() == std::mem::size_of::<i32>());
            try_valid_slice(caps.cast(), size)
        }
    }

    /// Gets the list of all SPIR-V extensions which were declared in the SPIR-V module.
    pub fn declared_extensions(&self) -> error::Result<ExtensionsIter<'static>> {
        // SAFETY: 'a is OK to return here
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_cross_c.cpp#L2756
        unsafe {
            let mut caps = std::ptr::null_mut();
            let mut size = 0;

            sys::spvc_compiler_get_declared_extensions(self.ptr.as_ptr(), &mut caps, &mut size)
                .ok(self)?;

            let ptr_slice = slice::from_raw_parts(caps, size);

            Ok(ExtensionsIter(ptr_slice.iter(), self.ctx.drop_guard()))
        }
    }

    /// Get the execution model of the module.
    pub fn execution_model(&self) -> error::Result<spirv::ExecutionModel> {
        unsafe {
            let exec_model = sys::spvc_compiler_get_execution_model(self.ptr.as_ptr());

            let Some(exec_model) = spirv::ExecutionModel::from_u32(exec_model.0 as u32) else {
                return Err(SpirvCrossError::InvalidEnum);
            };

            Ok(exec_model)
        }
    }
}

/// Proof that [`Compiler::update_active_builtins`] was called.
#[derive(Debug, Copy, Clone)]
pub struct ActiveBuiltinsUpdatedProof(Handle<()>);

/// Querying builtins in the SPIR-V module
impl<T> Compiler<T> {
    /// Update active built-ins in the SPIR-V module.
    pub fn update_active_builtins(&mut self) -> ActiveBuiltinsUpdatedProof {
        unsafe {
            sys::spvc_compiler_update_active_builtins(self.ptr.as_ptr());
            ActiveBuiltinsUpdatedProof(self.create_handle(()))
        }
    }

    /// Return whether the builtin is used or not.
    ///
    /// Requires [`Compiler::update_active_builtins`] to be called first,
    /// proof of which is required to call this function.
    pub fn has_active_builtin(
        &self,
        builtin: spirv::BuiltIn,
        storage_class: spirv::StorageClass,
        proof: ActiveBuiltinsUpdatedProof,
    ) -> error::Result<bool> {
        if !self.handle_is_valid(&proof.0) {
            return Err(SpirvCrossError::InvalidOperation(String::from(
                "The provided proof of building active builtins is invalid",
            )));
        }

        unsafe {
            Ok(sys::spvc_compiler_has_active_builtin(
                self.ptr.as_ptr(),
                SpvBuiltIn(builtin as i32),
                SpvStorageClass(storage_class as i32),
            ))
        }
    }
}

/// Iterator type created by [`Compiler::entry_points`].
pub struct EntryPointIter<'a>(slice::Iter<'a, spvc_entry_point>, AllocationDropGuard);

/// A SPIR-V entry point.
#[derive(Debug)]
pub struct EntryPoint<'a> {
    /// The execution model for the entry point.
    pub execution_model: spirv::ExecutionModel,
    /// The name of the entry point.
    pub name: CompilerStr<'a>,
}

impl_iterator!(EntryPointIter<'a>: EntryPoint<'a> as and_then|s, entry: &spvc_entry_point| {
    unsafe {
        let Some(execution_model) = spirv::ExecutionModel::from_u32(entry.execution_model.0 as u32) else {
            if cfg!(debug_assertions) {
                panic!("Unexpected SpvExecutionModelMax in valid entry point!")
            } else {
                return None;
            }
        };

        let name = CompilerStr::from_ptr(entry.name, s.1.clone());
        Some(EntryPoint {
            name,
            execution_model,
        })
    }
} for <'a> [0]);

/// Reflection of entry points.
impl<T> Compiler<T> {
    /// All operations work on the current entry point.
    ///
    /// Entry points can be swapped out with [`Compiler::set_entry_point`].
    ///
    /// Entry points should be set right after creating the compiler as some reflection
    /// functions traverse the graph from the entry point.
    ///
    /// Resource reflection also depends on the entry point.
    /// By default, the current entry point is set to the first `OpEntryPoint` which appears in the SPIR-V module.
    //
    /// Some shader languages restrict the names that can be given to entry points, and the
    /// corresponding backend will automatically rename an entry point name when compiling,
    /// if it is illegal.
    ///
    /// For example, the common entry point name `main()` is illegal in MSL, and is renamed to an
    /// alternate name by the MSL backend.
    ///
    /// Given the original entry point name contained in the SPIR-V, this function returns
    /// the name, as updated by the backend, if called after compilation.
    ///
    /// If the name is not illegal, and has not been renamed this function will simply return the
    /// original name.
    pub fn entry_points(&self) -> error::Result<EntryPointIter<'static>> {
        unsafe {
            // SAFETY: 'ctx is sound here
            // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_cross_c.cpp#L2170
            let mut entry_points = std::ptr::null();
            let mut size = 0;
            sys::spvc_compiler_get_entry_points(self.ptr.as_ptr(), &mut entry_points, &mut size)
                .ok(self)?;

            Ok(EntryPointIter(
                slice::from_raw_parts(entry_points.cast(), size).iter(),
                self.ctx.drop_guard(),
            ))
        }
    }

    /// Get the cleansed name of the entry point for the given original name.
    pub fn cleansed_entry_point_name<'str>(
        &self,
        name: impl Into<CompilerStr<'str>>,
        model: spirv::ExecutionModel,
    ) -> error::Result<Option<CompilerStr<'static>>> {
        // SAFETY: 'ctx is sound here
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/6a1fb66eef1bdca14acf7d0a51a3f883499d79f0/spirv_cross_c.cpp#L2217
        let name = name.into();
        let name = name.into_cstring_ptr()?;

        unsafe {
            let name = sys::spvc_compiler_get_cleansed_entry_point_name(
                self.ptr.as_ptr(),
                name.as_ptr(),
                SpvExecutionModel(model as u32 as i32),
            );

            if name.is_null() {
                return Ok(None);
            }
            Ok(Some(CompilerStr::from_ptr(name, self.ctx.drop_guard())))
        }
    }

    /// Set the current entry point by name.
    ///
    /// All operations work on the current entry point.
    ///
    /// Entry points should be set right after the constructor completes as some reflection functions traverse the graph from the entry point.
    /// Resource reflection also depends on the entry point.
    ///
    /// By default, the current entry point is set to the first OpEntryPoint which appears in the SPIR-V module.
    ///
    /// Names for entry points in the SPIR-V module may alias if they belong to different execution models.
    /// To disambiguate, we must pass along with the entry point names the execution model.
    ///
    /// ## Shader language restrictions
    /// Some shader languages restrict the names that can be given to entry points, and the
    /// corresponding backend will automatically rename an entry point name, on compilation if it is illegal.
    ///
    /// For example, the common entry point name `main()` is illegal in MSL, and is renamed to an
    /// alternate name by the MSL backend.
    pub fn set_entry_point<'str>(
        &mut self,
        name: impl Into<CompilerStr<'str>>,
        model: spirv::ExecutionModel,
    ) -> error::Result<()> {
        let name = name.into();
        unsafe {
            let name = name.into_cstring_ptr()?;

            sys::spvc_compiler_set_entry_point(
                self.ptr.as_ptr(),
                name.as_ptr(),
                SpvExecutionModel(model as u32 as i32),
            )
            .ok(&*self)
        }
    }

    /// Renames an entry point from `from` to `to`.
    ///
    /// If old_name is currently selected as the current entry point, it will continue to be the current entry point,
    /// albeit with a new name.
    ///
    /// Values returned from [`Compiler::entry_points`] before this call will be outdated.
    pub fn rename_entry_point<'str>(
        &mut self,
        from: impl Into<CompilerStr<'str>>,
        to: impl Into<CompilerStr<'str>>,
        model: spirv::ExecutionModel,
    ) -> error::Result<()> {
        let from = from.into();
        let to = to.into();

        unsafe {
            let from = from.into_cstring_ptr()?;
            let to = to.into_cstring_ptr()?;

            sys::spvc_compiler_rename_entry_point(
                self.ptr.as_ptr(),
                from.as_ptr(),
                to.as_ptr(),
                SpvExecutionModel(model as u32 as i32),
            )
            .ok(&*self)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::SpirvCrossError;
    use crate::Compiler;
    use crate::{targets, Module};
    use spirv::ExecutionModel;

    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn get_entry_points() -> Result<(), SpirvCrossError> {
        let vec = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&vec));

        let mut compiler: Compiler<targets::None> = Compiler::new(words)?;
        let old_entry_points: Vec<_> = compiler.entry_points()?.collect();
        let main = &old_entry_points[0];

        eprintln!("{:?}", main);

        assert_eq!("main", main.name.as_ref());
        compiler.rename_entry_point("main", "new_main", spirv::ExecutionModel::Fragment)?;

        let no_name =
            compiler.cleansed_entry_point_name("main", spirv::ExecutionModel::Fragment)?;

        assert!(no_name.is_none());

        assert_eq!("main", main.name.as_ref());
        let new_name =
            compiler.cleansed_entry_point_name("new_main", spirv::ExecutionModel::Fragment)?;

        assert_eq!(Some("new_main"), new_name.as_deref());

        Ok(())
    }

    #[test]
    pub fn entry_point_soundness() -> Result<(), SpirvCrossError> {
        let vec = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&vec));

        let mut compiler: Compiler<targets::None> = Compiler::new(words)?;
        let entry_points = compiler.entry_points()?;
        let name = compiler
            .cleansed_entry_point_name("main", spirv::ExecutionModel::Fragment)?
            .unwrap();

        assert_eq!("main", name.as_ref());

        drop(compiler);

        assert_eq!("main", name.as_ref());
        let entries: Vec<_> = entry_points.collect();

        eprintln!("{:?}", entries);
        Ok(())
    }

    #[test]
    pub fn capabilities() -> Result<(), SpirvCrossError> {
        let vec = Vec::from(BASIC_SPV);
        let words = Module::from_words(bytemuck::cast_slice(&vec));

        let mut compiler: Compiler<targets::None> = Compiler::new(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        let ty = compiler.declared_capabilities()?;

        assert_eq!([spirv::Capability::Shader], ty);

        Ok(())
    }
}
