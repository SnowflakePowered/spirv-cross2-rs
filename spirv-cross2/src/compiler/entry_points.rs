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


// SPVC_PUBLIC_API spvc_result spvc_compiler_get_declared_capabilities(spvc_compiler compiler,
// const SpvCapability **capabilities,
// size_t *num_capabilities);
// SPVC_PUBLIC_API spvc_result spvc_compiler_get_declared_extensions(spvc_compiler compiler, const char ***extensions,
// size_t *num_extensions);
//

use crate::compiler::Compiler;

impl <T> Compiler<'_, T> {


}