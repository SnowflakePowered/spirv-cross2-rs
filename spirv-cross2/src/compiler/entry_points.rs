use core::slice;
use spirv_cross_sys as sys;
use spirv_cross_sys::spvc_entry_point;
use std::ffi::{c_char, CStr};
use std::panic::catch_unwind;
// TODO:
// SPVC_PUBLIC_API spvc_result spvc_compiler_get_entry_points(spvc_compiler compiler,
// const spvc_entry_point **entry_points,
// size_t *num_entry_points);
// SPVC_PUBLIC_API spvc_result spvc_compiler_set_entry_point(spvc_compiler compiler, const char *name,
// SpvExecutionModel model);
// SPVC_PUBLIC_API spvc_result spvc_compiler_rename_entry_point(spvc_compiler compiler, const char *old_name,
// const char *new_name, SpvExecutionModel model);
// SPVC_PUBLIC_API const char *spvc_compiler_get_cleansed_entry_point_name(spvc_compiler compiler, const char *name,
// SpvExecutionModel model);

// SPVC_PUBLIC_API void spvc_compiler_set_execution_mode(spvc_compiler compiler, SpvExecutionMode mode);
// SPVC_PUBLIC_API void spvc_compiler_unset_execution_mode(spvc_compiler compiler, SpvExecutionMode mode);
// SPVC_PUBLIC_API void spvc_compiler_set_execution_mode_with_arguments(spvc_compiler compiler, SpvExecutionMode mode,
// unsigned arg0, unsigned arg1, unsigned arg2);
// SPVC_PUBLIC_API spvc_result spvc_compiler_get_execution_modes(spvc_compiler compiler, const SpvExecutionMode **modes,
// size_t *num_modes);
// SPVC_PUBLIC_API unsigned spvc_compiler_get_execution_mode_argument(spvc_compiler compiler, SpvExecutionMode mode);
// SPVC_PUBLIC_API unsigned spvc_compiler_get_execution_mode_argument_by_index(spvc_compiler compiler,
// SpvExecutionMode mode, unsigned index);
// SPVC_PUBLIC_API SpvExecutionModel spvc_compiler_get_execution_model(spvc_compiler compiler);
// SPVC_PUBLIC_API void spvc_compiler_update_active_builtins(spvc_compiler compiler);
// SPVC_PUBLIC_API spvc_bool spvc_compiler_has_active_builtin(spvc_compiler compiler, SpvBuiltIn builtin, SpvStorageClass storage);

use crate::compiler::{Compiler, PhantomCompiler};
use crate::error::{SpirvCrossError, ToContextError};
use crate::{error, spirv};
use crate::string::MaybeCStr;

pub struct ExtensionsIter<'a>(slice::Iter<'a, *const c_char>);

impl<'a> Iterator for ExtensionsIter<'a> {
    type Item = MaybeCStr<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|ptr| unsafe { MaybeCStr::from_ptr(*ptr) })
    }
}

/// Querying declared properties of the SPIR-V module.
impl<'a, T> Compiler<'a, T> {
    /// Gets the list of all SPIR-V Capabilities which were declared in the SPIR-V module.
    pub fn declared_capabilities(&self) -> error::Result<&'a [spirv::Capability]> {
        unsafe {
            let mut caps = std::ptr::null();
            let mut size = 0;

            sys::spvc_compiler_get_declared_capabilities(self.ptr.as_ptr(), &mut caps, &mut size)
                .ok(self)?;

            Ok(slice::from_raw_parts(caps, size))
        }
    }

    /// Gets the list of all SPIR-V extensions which were declared in the SPIR-V module.
    pub fn declared_extensions(&self) -> error::Result<ExtensionsIter<'a>> {
        unsafe {
            let mut caps = std::ptr::null_mut();
            let mut size = 0;

            sys::spvc_compiler_get_declared_extensions(self.ptr.as_ptr(), &mut caps, &mut size)
                .ok(self)?;

            let ptr_slice = slice::from_raw_parts(caps, size);

            Ok(ExtensionsIter(ptr_slice.into_iter()))
        }
    }

    /// Get the execution model of the module.
    pub fn execution_model(&self) -> spirv::ExecutionModel {
        unsafe { sys::spvc_compiler_get_execution_model(self.ptr.as_ptr()) }
    }
}

pub struct EntryPointIter<'a>(slice::Iter<'a, spvc_entry_point>);

#[derive(Debug)]
pub struct EntryPoint<'a> {
    pub execution_model: spirv::ExecutionModel,
    pub name: MaybeCStr<'a>,
}

impl<'a> Iterator for EntryPointIter<'a> {
    type Item = EntryPoint<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|entry| unsafe {
            let name = MaybeCStr::from_ptr(entry.name);
            EntryPoint {
                name,
                execution_model: entry.execution_model,
            }
        })
    }
}

/// Reflection of entry points.
impl<'a, T> Compiler<'a, T> {
    /// All operations work on the current entry point.
    /// Entry points can be swapped out with set_entry_point().
    /// Entry points should be set right after the constructor completes as some reflection functions traverse the graph from the entry point.
    /// Resource reflection also depends on the entry point.
    /// By default, the current entry point is set to the first OpEntryPoint which appears in the SPIR-V module.
    //
    /// Some shader languages restrict the names that can be given to entry points, and the
    /// corresponding backend will automatically rename an entry point name, during the call
    /// to compile() if it is illegal. For example, the common entry point name main() is
    /// illegal in MSL, and is renamed to an alternate name by the MSL backend.
    /// Given the original entry point name contained in the SPIR-V, this function returns
    /// the name, as updated by the backend during the call to compile(). If the name is not
    /// illegal, and has not been renamed, or if this function is called before compile(),
    /// this function will simply return the same name.
    //
    /// New variants of entry point query and reflection.
    /// Names for entry points in the SPIR-V module may alias if they belong to different execution models.
    /// To disambiguate, we must pass along with the entry point names the execution model.
    pub fn entry_points(&self) -> error::Result<EntryPointIter<'a>> {
        unsafe {
            let mut entry_points = std::ptr::null();
            let mut size = 0;
            sys::spvc_compiler_get_entry_points(self.ptr.as_ptr(), &mut entry_points, &mut size)
                .ok(self)?;

            Ok(EntryPointIter(
                slice::from_raw_parts(entry_points, size).into_iter(),
            ))
        }
    }

    /// Get the
    pub fn cleansed_entry_point_name<'str>(
        &self,
        name: impl Into<MaybeCStr<'str>>,
        model: spirv::ExecutionModel,
    ) -> error::Result<Option<MaybeCStr<'a>>> {
        let name = name.into();

        unsafe {
            let Ok(name) = name.to_cstring_ptr() else {
                return Err(SpirvCrossError::InvalidName(name.to_string()));
            };

            let name = sys::spvc_compiler_get_cleansed_entry_point_name(
                self.ptr.as_ptr(),
                name.as_ptr(),
                model,
            );

            if name == std::ptr::null() {
                return Ok(None);
            }
            Ok(Some(MaybeCStr::from_ptr(name)))
        }
    }

    pub fn set_entry_point<'str>(
        &mut self,
        name: impl Into<MaybeCStr<'str>>,
        model: spirv::ExecutionModel,
    ) -> error::Result<()> {
        let name = name.into();
        unsafe {
            let Ok(name) = name.to_cstring_ptr() else {
                return Err(SpirvCrossError::InvalidName(name.to_string()));
            };

            sys::spvc_compiler_set_entry_point_safe(self.ptr.as_ptr(), name.as_ptr(), model)
                .ok(&*self)
        }
    }

    pub fn rename_entry_point<'str>(
        &mut self,
        from: impl Into<MaybeCStr<'str>>,
        to: impl Into<MaybeCStr<'str>>,
        model: spirv::ExecutionModel,
    ) -> error::Result<()> {
        let from = from.into();
        let to = to.into();

        unsafe {
            let Ok(from) = from.to_cstring_ptr() else {
                return Err(SpirvCrossError::InvalidName(from.as_ref().to_string()));
            };

            let Ok(to) = to.to_cstring_ptr() else {
                return Err(SpirvCrossError::InvalidName(to.as_ref().to_string()));
            };

            sys::spvc_compiler_rename_entry_point(
                self.ptr.as_ptr(),
                from.as_ptr(),
                to.as_ptr(),
                model,
            )
            .ok(&*self)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::Compiler;
    use crate::error::SpirvCrossError;
    use crate::{spirv, targets, Module, SpirvCross};

    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn get_entry_points() -> Result<(), SpirvCrossError> {
        let mut spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let mut compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let entry_points: Vec<_> = compiler.entry_points()?.collect();
        let main = &entry_points[0];


        eprintln!("{entry_points:?}");
        compiler.set_entry_point("not_main", spirv::ExecutionModel::Fragment)?;

        let new_name = compiler.cleansed_entry_point_name("main", spirv::ExecutionModel::Fragment)?;

        eprintln!("{:?}", new_name);
        eprintln!("{entry_points:?}");
        //
        // let entry_points: Vec<_> = compiler.entry_points()?.collect();
        // let main = &entry_points[0];
        Ok(())
    }
}

