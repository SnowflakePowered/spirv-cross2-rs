use bindgen::callbacks::{EnumVariantCustomBehavior, EnumVariantValue, ParseCallbacks};
use cruet::Inflector;

#[derive(Debug)]
pub struct SpirvCrossCallbacks;

const SPVC_HLSL_TYPES: &[&str] = &[
    "spvc_hlsl_binding_flag_bits",
    "spvc_hlsl_resource_binding",
    "spvc_hlsl_resource_binding_mapping",
    "spvc_hlsl_root_constants",
    "spvc_hlsl_vertex_attribute_remap",
    "spvc_hlsl_binding_flags",
];

const SPVC_MSL_TYPES: &[&str] = &[
    "spvc_msl_constexpr_sampler",
    "spvc_msl_resource_binding",
    "spvc_msl_resource_binding_2",
    "spvc_msl_sampler_ycbcr_conversion",
    "spvc_msl_shader_interface_var",
    "spvc_msl_shader_interface_var_2",
    "spvc_msl_vertex_attribute",
    "spvc_msl_chroma_location",
    "spvc_msl_component_swizzle",
    "spvc_msl_format_resolution",
    "spvc_msl_index_type",
    "spvc_msl_platform",
    "spvc_msl_sampler_address",
    "spvc_msl_sampler_border_color",
    "spvc_msl_sampler_compare_func",
    "spvc_msl_sampler_coord",
    "spvc_msl_sampler_filter",
    "spvc_msl_sampler_mip_filter",
    "spvc_msl_sampler_ycbcr_model_conversion",
    "spvc_msl_sampler_ycbcr_range",
    "spvc_msl_shader_input_format",
    "spvc_msl_shader_variable_format",
    "spvc_msl_shader_variable_rate",
    "spvc_msl_vertex_format",
    "spvc_msl_shader_input",
];

impl ParseCallbacks for SpirvCrossCallbacks {
    fn item_name(&self, original_item_name: &str) -> Option<String> {
        // Keep the Spv prefix cause bindgen can't namespace.
        // in the Rust wrapper, we can make it nicer.
        if original_item_name.starts_with("Spv") {
            // strip the suffix.
            if let Some(name) = original_item_name.strip_suffix("_") {
                return Some(name.to_string());
            };
        }

        if let Some(name) = original_item_name.strip_prefix("spvc_msl_") {
            if SPVC_MSL_TYPES.contains(&original_item_name) {
                let name = name.to_pascal_case();
                return Some(format!("Msl{name}"));
            }
        }

        if let Some(name) = original_item_name.strip_prefix("spvc_hlsl_") {
            if SPVC_HLSL_TYPES.contains(&original_item_name) {
                let name = name.to_pascal_case();
                return Some(format!("Hlsl{name}"));
            }
        }

        Some(String::from(match original_item_name {
            "spvc_constant_id" => "ConstantId",
            "spvc_type_id" => "TypeId",
            "spvc_variable_id" => "VariableId",
            "spvc_buffer_range" => "BufferRange",
            "spvc_backend" => "CompilerBackend",
            "spvc_basetype" => "BaseType",
            "spvc_builtin_resource_type" => "BuiltinResourceType",
            "spvc_capture_mode" => "CaptureMode",
            "spvc_resource_type" => "ResourceType",
            "spvc_combined_image_sampler" => "CombinedImageSampler",

            // While `spvc_bool` is typedefed to `unsigned char`, it is always `stdbool`, which is ABI compatible with Rust's `bool`
            "spvc_bool" => "crate::spvc_bool",
            _ => return None,
        }))
    }

    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<String> {
        if let Some(enum_name) = enum_name
            .and_then(|n| n.strip_prefix("enum "))
            .and_then(|n| n.strip_suffix("_"))
        {
            // strip prefix
            if let Some(name) = original_variant_name
                .strip_prefix(enum_name)
                .map(|s| s.to_string())
            {
                // Special case SpvDim
                if original_variant_name.starts_with("SpvDim") {
                    return Some(format!("Dim{name}"));
                }

                return Some(name);
            }
        };

        if let Some(enum_name) = enum_name.and_then(|n| n.strip_prefix("enum ")) {
            // strip prefix
            if enum_name.starts_with("spvc_") {
                // eprintln!("{original_variant_name}");
                if let Some(name) = original_variant_name
                    .strip_prefix(&format!("{}_", enum_name.to_ascii_uppercase()))
                {
                    // eprintln!("{}", enum_name);
                    // Special case FormatResolution
                    if original_variant_name.starts_with("SPVC_MSL_FORMAT_RESOLUTION") {
                        return Some(format!("FormatResolution{name}"));
                    }

                    return Some(format!("{}", name.to_pascal_case()));
                }
            }
        };

        return None;
    }

    fn enum_variant_behavior(
        &self,
        _enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<EnumVariantCustomBehavior> {
        if original_variant_name.ends_with("_MAX") {
            return Some(EnumVariantCustomBehavior::Hide);
        };

        if original_variant_name.ends_with("Max") {
            return Some(EnumVariantCustomBehavior::Hide);
        };

        if original_variant_name.ends_with("MaxInt") {
            return Some(EnumVariantCustomBehavior::Hide);
        };

        if original_variant_name.ends_with("_MAX_INT") {
            return Some(EnumVariantCustomBehavior::Hide);
        };

        return None;
    }
}
