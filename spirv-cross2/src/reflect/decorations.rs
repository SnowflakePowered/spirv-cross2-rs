use crate::error::{SpirvCrossError, ToContextError};
use crate::handle::{Handle, Id};
use crate::reflect::{Resource, StructMember};
use crate::sealed::Sealed;
use crate::spirv::Decoration;
use crate::string::ContextStr;
use crate::Compiler;
use crate::{error, spirv, ToStatic};
use core::slice;
use spirv_cross_sys as sys;
use spirv_cross_sys::{BaseType, ConstantId, FromPrimitive, SpvId, ToPrimitive, VariableId};

#[derive(Debug)]
pub enum DecorationValue<'a> {
    /// Returned by the following decorations.
    ///
    /// - [`Location`](Decoration::Location).
    /// - [`Component`](Decoration::Component).
    /// - [`Offset`](Decoration::Offset).
    /// - [`XfbBuffer`](Decoration::XfbBuffer).
    /// - [`XfbStride`](Decoration::XfbStride).
    /// - [`Stream`](Decoration::Stream).
    /// - [`Binding`](Decoration::Binding).
    /// - [`DescriptorSet`](Decoration::DescriptorSet).
    /// - [`InputAttachmentIndex`](Decoration::InputAttachmentIndex).
    /// - [`ArrayStride`](Decoration::ArrayStride).
    /// - [`MatrixStride`](Decoration::MatrixStride).
    /// - [`Index`](Decoration::Index).
    Literal(u32),
    /// Only for decoration [`BuiltIn`](Decoration::BuiltIn).
    BuiltIn(spirv::BuiltIn),
    /// Only for decoration [`FPRoundingMode`](Decoration::FPRoundingMode).
    RoundingMode(spirv::FPRoundingMode),
    /// Only for decoration [`SpecId`](Decoration::SpecId).
    Constant(Handle<ConstantId>),
    /// Only for decoration [`HlslSemanticGOOGLE`](Decoration::HlslSemanticGOOGLE) and [`UserTypeGOOGLE`](Decoration::HlslSemanticGOOGLE).
    String(ContextStr<'a>),
    /// All other decorations to indicate the presence of a decoration.
    Present,
}

impl Sealed for DecorationValue<'_> {}
impl ToStatic for DecorationValue<'_> {
    type Static<'a>
    = DecorationValue<'static>
    where
        'a: 'static;

    fn to_static(&self) -> Self::Static<'static> {
        match self {
            DecorationValue::Literal(a) => DecorationValue::Literal(*a),
            DecorationValue::BuiltIn(a) => DecorationValue::BuiltIn(*a),
            DecorationValue::RoundingMode(a) => DecorationValue::RoundingMode(*a),
            DecorationValue::Constant(a) => DecorationValue::Constant(*a),
            DecorationValue::String(c) => {
                let owned = c.to_string();
                DecorationValue::String(ContextStr::from_string(owned))
            }
            DecorationValue::Present => DecorationValue::Present,
        }
    }
}

impl<'a> Clone for DecorationValue<'a> {
    fn clone(&self) -> DecorationValue<'static> {
        self.to_static()
    }
}

impl DecorationValue<'_> {
    /// Check that the value is valid for the decoration type.
    pub fn type_is_valid_for_decoration(&self, decoration: Decoration) -> bool {
        match self {
            DecorationValue::Literal(_) => decoration_is_literal(decoration),
            DecorationValue::BuiltIn(_) => decoration == Decoration::BuiltIn,
            DecorationValue::RoundingMode(_) => decoration == Decoration::FPRoundingMode,
            DecorationValue::Constant(_) => decoration == Decoration::SpecId,
            DecorationValue::String(_) => decoration_is_string(decoration),
            DecorationValue::Present => {
                !decoration_is_literal(decoration)
                    && !decoration_is_string(decoration)
                    && decoration != Decoration::BuiltIn
                    && decoration != Decoration::FPRoundingMode
                    && decoration != Decoration::SpecId
            }
        }
    }
}
fn decoration_is_literal(decoration: Decoration) -> bool {
    match decoration {
        Decoration::Location
        | Decoration::Component
        | Decoration::Offset
        | Decoration::XfbBuffer
        | Decoration::XfbStride
        | Decoration::Stream
        | Decoration::Binding
        | Decoration::DescriptorSet
        | Decoration::InputAttachmentIndex
        | Decoration::ArrayStride
        | Decoration::MatrixStride
        | Decoration::Index => true,
        _ => false,
    }
}

fn decoration_is_string(decoration: Decoration) -> bool {
    match decoration {
        Decoration::HlslSemanticGOOGLE | Decoration::UserTypeGOOGLE => true,
        _ => false,
    }
}

impl<'a, T> Compiler<'a, T> {
    /// Gets the value for decorations which take arguments.
    pub fn decoration<I: Id>(
        &self,
        id: Handle<I>,
        decoration: Decoration,
    ) -> error::Result<Option<DecorationValue>> {
        // SAFETY: id is yielded by the instance so it's safe to use.
        let id = SpvId(self.yield_id(id)?.id());
        unsafe {
            let has_decoration =
                sys::spvc_compiler_has_decoration(self.ptr.as_ptr(), id, decoration);
            if !has_decoration {
                return Ok(None);
            };

            if decoration_is_string(decoration) {
                let str =
                    sys::spvc_compiler_get_decoration_string(self.ptr.as_ptr(), id, decoration);
                return Ok(Some(DecorationValue::String(ContextStr::from_ptr(str))));
            }

            let value = sys::spvc_compiler_get_decoration(self.ptr.as_ptr(), id, decoration);
            self.parse_decoration_value(decoration, value)
        }
    }

    /// Gets the value for member decorations which take arguments.
    pub fn member_decoration<I: Id>(
        &self,
        member: &StructMember<'a>,
        decoration: Decoration,
    ) -> error::Result<Option<DecorationValue>> {
        // SAFETY: id is yielded by the instance so it's safe to use.
        let struct_type = self.yield_id(member.struct_type)?;
        let index = member.index as u32;

        unsafe {
            let has_decoration = sys::spvc_compiler_has_member_decoration(
                self.ptr.as_ptr(),
                struct_type,
                index,
                decoration,
            );
            if !has_decoration {
                return Ok(None);
            };

            if decoration_is_string(decoration) {
                let str = sys::spvc_compiler_get_member_decoration_string(
                    self.ptr.as_ptr(),
                    struct_type,
                    index,
                    decoration,
                );
                return Ok(Some(DecorationValue::String(ContextStr::from_ptr(str))));
            }

            let value = sys::spvc_compiler_get_member_decoration(
                self.ptr.as_ptr(),
                struct_type,
                index,
                decoration,
            );
            self.parse_decoration_value(decoration, value)
        }
    }

    /// Set the value of a decoration for an ID.
    pub fn set_decoration<'value, I: Id>(
        &mut self,
        id: Handle<I>,
        decoration: Decoration,
        value: Option<DecorationValue<'value>>,
    ) -> error::Result<()>
    where
        'value: 'a,
    {
        // SAFETY: id is yielded by the instance so it's safe to use.
        let id = SpvId(self.yield_id(id)?.id());
        unsafe {
            let Some(value) = value else {
                sys::spvc_compiler_unset_decoration(self.ptr.as_ptr(), id, decoration);
                return Ok(());
            };

            if !value.type_is_valid_for_decoration(decoration) {
                return Err(SpirvCrossError::InvalidDecorationInput(
                    decoration,
                    DecorationValue::to_static(&value),
                ));
            }

            match value {
                DecorationValue::Literal(literal) => {
                    sys::spvc_compiler_set_decoration(self.ptr.as_ptr(), id, decoration, literal);
                }
                DecorationValue::BuiltIn(builtin) => {
                    let Some(builtin) = builtin.to_u32() else {
                        return Err(SpirvCrossError::InvalidDecorationInput(
                            decoration,
                            DecorationValue::to_static(&value),
                        ));
                    };

                    sys::spvc_compiler_set_decoration(self.ptr.as_ptr(), id, decoration, builtin);
                }
                DecorationValue::RoundingMode(rounding_mode) => {
                    let Some(rounding_mode) = rounding_mode.to_u32() else {
                        return Err(SpirvCrossError::InvalidDecorationInput(
                            decoration,
                            DecorationValue::to_static(&value),
                        ));
                    };

                    sys::spvc_compiler_set_decoration(
                        self.ptr.as_ptr(),
                        id,
                        decoration,
                        rounding_mode,
                    );
                }
                DecorationValue::Constant(constant) => {
                    let constant = self.yield_id(constant)?;
                    sys::spvc_compiler_set_decoration(
                        self.ptr.as_ptr(),
                        id,
                        decoration,
                        constant.id(),
                    );
                }
                DecorationValue::Present => {
                    sys::spvc_compiler_set_decoration(self.ptr.as_ptr(), id, decoration, 1);
                }
                DecorationValue::String(ref string) => {
                    let Ok(cstring) = string.to_cstring_ptr() else {
                        return Err(SpirvCrossError::InvalidDecorationInput(
                            decoration,
                            DecorationValue::to_static(&value),
                        ));
                    };

                    sys::spvc_compiler_set_decoration_string(
                        self.ptr.as_ptr(),
                        id,
                        decoration,
                        cstring.as_ptr(),
                    );

                    // Sanity drop to show that the lifetime of the cstring is only up until
                    // we have returned. AFAIK, SPIRV-Cross will do a string copy.
                    // If it does not, then we'll have to keep this string alive for a while.
                    drop(cstring);
                }
            }
        }
        Ok(())
    }

    /// Set the value of a decoration for a struct member.
    pub fn set_member_decoration<'value>(
        &mut self,
        member: &StructMember<'a>,
        decoration: Decoration,
        value: Option<DecorationValue<'value>>,
    ) -> error::Result<()>
    where
        'value: 'a,
    {
        // SAFETY: id is yielded by the instance so it's safe to use.
        let struct_type = self.yield_id(member.struct_type)?;
        let index = member.index as u32;

        unsafe {
            let Some(value) = value else {
                sys::spvc_compiler_unset_member_decoration(
                    self.ptr.as_ptr(),
                    struct_type,
                    index,
                    decoration,
                );
                return Ok(());
            };

            if !value.type_is_valid_for_decoration(decoration) {
                return Err(SpirvCrossError::InvalidDecorationInput(
                    decoration,
                    DecorationValue::to_static(&value),
                ));
            }

            match value {
                DecorationValue::Literal(literal) => {
                    sys::spvc_compiler_set_member_decoration(
                        self.ptr.as_ptr(),
                        struct_type,
                        index,
                        decoration,
                        literal,
                    );
                }
                DecorationValue::BuiltIn(builtin) => {
                    let Some(builtin) = builtin.to_u32() else {
                        return Err(SpirvCrossError::InvalidDecorationInput(
                            decoration,
                            DecorationValue::to_static(&value),
                        ));
                    };

                    sys::spvc_compiler_set_member_decoration(
                        self.ptr.as_ptr(),
                        struct_type,
                        index,
                        decoration,
                        builtin,
                    );
                }
                DecorationValue::RoundingMode(rounding_mode) => {
                    let Some(rounding_mode) = rounding_mode.to_u32() else {
                        return Err(SpirvCrossError::InvalidDecorationInput(
                            decoration,
                            DecorationValue::to_static(&value),
                        ));
                    };

                    sys::spvc_compiler_set_member_decoration(
                        self.ptr.as_ptr(),
                        struct_type,
                        index,
                        decoration,
                        rounding_mode,
                    );
                }
                DecorationValue::Constant(constant) => {
                    let constant = self.yield_id(constant)?;
                    sys::spvc_compiler_set_member_decoration(
                        self.ptr.as_ptr(),
                        struct_type,
                        index,
                        decoration,
                        constant.id(),
                    );
                }
                DecorationValue::Present => {
                    sys::spvc_compiler_set_member_decoration(
                        self.ptr.as_ptr(),
                        struct_type,
                        index,
                        decoration,
                        1,
                    );
                }
                DecorationValue::String(ref string) => {
                    let Ok(cstring) = string.to_cstring_ptr() else {
                        return Err(SpirvCrossError::InvalidDecorationInput(
                            decoration,
                            DecorationValue::to_static(&value),
                        ));
                    };

                    sys::spvc_compiler_set_member_decoration_string(
                        self.ptr.as_ptr(),
                        struct_type,
                        index,
                        decoration,
                        cstring.as_ptr(),
                    );

                    // Sanity drop to show that the lifetime of the cstring is only up until
                    // we have returned. AFAIK, SPIRV-Cross will do a string copy.
                    // If it does not, then we'll have to keep this string alive for a while.
                    drop(cstring);
                }
            }
        }
        Ok(())
    }

    /// Gets the offset in SPIR-V words (uint32_t) for a decoration which was originally declared in the SPIR-V binary.
    /// The offset will point to one or more uint32_t literals which can be modified in-place before using the SPIR-V binary.
    ///
    /// Note that adding or removing decorations using the reflection API will not change the behavior of this function.
    /// If the decoration was declared, returns an offset into the provided SPIR-V binary buffer,
    /// otherwise returns None.
    ///
    /// If the decoration does not have any value attached to it (e.g. DecorationRelaxedPrecision), this function will also return None.
    pub fn binary_offset_for_decoration(
        &self,
        variable: impl Into<Handle<VariableId>>,
        decoration: Decoration,
    ) -> error::Result<Option<u32>> {
        let id = self.yield_id(variable.into())?;

        unsafe {
            let mut offset = 0;
            if !sys::spvc_compiler_get_binary_offset_for_decoration(
                self.ptr.as_ptr(),
                id,
                decoration,
                &mut offset,
            ) {
                Ok(None)
            } else {
                Ok(Some(offset))
            }
        }
    }

    fn parse_decoration_value(
        &self,
        decoration: Decoration,
        value: u32,
    ) -> error::Result<Option<DecorationValue>> {
        if decoration_is_literal(decoration) {
            return Ok(Some(DecorationValue::Literal(value)));
        }

        // String is handled.
        match decoration {
            Decoration::BuiltIn => {
                let Some(builtin) = spirv::BuiltIn::from_u32(value) else {
                    return Err(SpirvCrossError::InvalidDecorationOutput(decoration, value));
                };
                Ok(Some(DecorationValue::BuiltIn(builtin)))
            }
            Decoration::FPRoundingMode => {
                let Some(rounding_mode) = spirv::FPRoundingMode::from_u32(value) else {
                    return Err(SpirvCrossError::InvalidDecorationOutput(decoration, value));
                };
                Ok(Some(DecorationValue::RoundingMode(rounding_mode)))
            }
            Decoration::SpecId => unsafe {
                Ok(Some(DecorationValue::Constant(
                    self.create_handle(ConstantId(SpvId(value))),
                )))
            },
            _ => {
                if value == 1 {
                    Ok(Some(DecorationValue::Present))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Get the decorations for a buffer block resource.
    ///
    /// If the resource is not a struct, returns None.
    pub fn buffer_block_decorations(
        &self,
        block: &Resource<'a>,
    ) -> error::Result<Option<&'a [Decoration]>> {
        // this API is weird because there's an assert in
        // `get_block_buffer_flags`
        // https://github.com/KhronosGroup/SPIRV-Cross/blob/main/spirv_cross_parsed_ir.cpp#L567
        // that is not caught by the C API.
        // so we need to pre-check the type.

        let id = self.yield_id(block.id)?;
        let base_type_id = self.yield_id(block.base_type_id)?;

        unsafe {
            let ty = sys::spvc_compiler_get_type_handle(self.ptr.as_ptr(), base_type_id);

            let base_ty = sys::spvc_type_get_basetype(ty);

            if base_ty != BaseType::Struct {
                return Ok(None);
            }

            let mut size = 0;
            let mut buffer = std::ptr::null();
            sys::spvc_compiler_get_buffer_block_decorations(
                self.ptr.as_ptr(),
                id,
                &mut buffer,
                &mut size,
            )
            .ok(self)?;

            let slice = slice::from_raw_parts(buffer, size);
            if slice.is_empty() {
                Ok(None)
            } else {
                Ok(Some(slice))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::SpirvCrossError;
    use crate::Compiler;

    use crate::{targets, Module, SpirvCross};

    static BASIC_SPV: &[u8] = include_bytes!("../../basic.spv");

    #[test]
    pub fn set_decoration_test() -> Result<(), SpirvCrossError> {
        let spv = SpirvCross::new()?;
        let words = Module::from_words(bytemuck::cast_slice(BASIC_SPV));

        let compiler: Compiler<targets::None> = spv.create_compiler(words)?;
        let resources = compiler.shader_resources()?.all_resources()?;

        // compiler.set_decoration(Decoration::HlslSemanticGOOGLE, DecorationValue::String(Cow::Borrowed("hello")));
        Ok(())
    }
}
