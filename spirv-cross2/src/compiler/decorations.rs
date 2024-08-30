// TODO:
// SPVC_PUBLIC_API void spvc_compiler_set_decoration(spvc_compiler compiler, SpvId id, SpvDecoration decoration,
// unsigned argument);
// SPVC_PUBLIC_API void spvc_compiler_set_decoration_string(spvc_compiler compiler, SpvId id, SpvDecoration decoration,
// const char *argument);
// SPVC_PUBLIC_API void spvc_compiler_set_name(spvc_compiler compiler, SpvId id, const char *argument);
// SPVC_PUBLIC_API void spvc_compiler_set_member_decoration(spvc_compiler compiler, spvc_type_id id, unsigned member_index,
// SpvDecoration decoration, unsigned argument);
// SPVC_PUBLIC_API void spvc_compiler_set_member_decoration_string(spvc_compiler compiler, spvc_type_id id,
// unsigned member_index, SpvDecoration decoration,
// const char *argument);
// SPVC_PUBLIC_API void spvc_compiler_set_member_name(spvc_compiler compiler, spvc_type_id id, unsigned member_index,
// const char *argument);
// SPVC_PUBLIC_API void spvc_compiler_unset_decoration(spvc_compiler compiler, SpvId id, SpvDecoration decoration);
// SPVC_PUBLIC_API void spvc_compiler_unset_member_decoration(spvc_compiler compiler, spvc_type_id id,
// unsigned member_index, SpvDecoration decoration);
//
// SPVC_PUBLIC_API spvc_bool spvc_compiler_has_decoration(spvc_compiler compiler, SpvId id, SpvDecoration decoration);
// SPVC_PUBLIC_API spvc_bool spvc_compiler_has_member_decoration(spvc_compiler compiler, spvc_type_id id,
// unsigned member_index, SpvDecoration decoration);
// SPVC_PUBLIC_API const char *spvc_compiler_get_name(spvc_compiler compiler, SpvId id);
// SPVC_PUBLIC_API unsigned spvc_compiler_get_decoration(spvc_compiler compiler, SpvId id, SpvDecoration decoration);
// SPVC_PUBLIC_API const char *spvc_compiler_get_decoration_string(spvc_compiler compiler, SpvId id,
// SpvDecoration decoration);
// SPVC_PUBLIC_API unsigned spvc_compiler_get_member_decoration(spvc_compiler compiler, spvc_type_id id,
// unsigned member_index, SpvDecoration decoration);
// SPVC_PUBLIC_API const char *spvc_compiler_get_member_decoration_string(spvc_compiler compiler, spvc_type_id id,
// unsigned member_index, SpvDecoration decoration);
// SPVC_PUBLIC_API const char *spvc_compiler_get_member_name(spvc_compiler compiler, spvc_type_id id, unsigned member_index);
//
// SPVC_PUBLIC_API spvc_bool spvc_compiler_get_binary_offset_for_decoration(spvc_compiler compiler,
// spvc_variable_id id,
// SpvDecoration decoration,
// unsigned *word_offset);