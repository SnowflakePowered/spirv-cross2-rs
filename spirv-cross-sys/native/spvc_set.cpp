#include "spirv_cross_c.h"
#include <cstdint>
#include <unordered_set>

// This is a hack to get the expected ABI for spvc_set.
struct __Internal_ScratchMemoryAllocation
{
	virtual ~__Internal_ScratchMemoryAllocation() = default;
};

struct __internal_spvc_set_s : __Internal_ScratchMemoryAllocation
{
	std::unordered_set<uint32_t> set;
};

extern "C" void spvc_rs_expose_set(spvc_set opaque_set, uint32_t* out, size_t* length) {
    // Extremely important that opaque_set is always accessed via pointer, to avoid triggering RAII.
    const __internal_spvc_set_s *set = reinterpret_cast<const __internal_spvc_set_s *>(opaque_set);
    if (length != nullptr) {
        *length = set->set.size();
    }

    if (out == nullptr) {
        return;
    }

    for (auto &id: set->set) {
       *out = id;
       out++;
    }
}