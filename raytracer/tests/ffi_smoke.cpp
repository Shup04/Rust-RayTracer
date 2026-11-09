#include <cstdlib>
#include <cmath>
#include <cstdint>
#include <iostream>
#include <vector>
#include "raytracer_ffi.h"

static std::vector<unsigned char> snapshot() {
    const auto* begin = reinterpret_cast<const unsigned char*>(get_image_ptr());
    return {begin, begin + get_image_width() * get_image_height() * 4};
}

static void require(bool condition) {
    if (!condition) {
        std::cerr << "FFI smoke assertion failed\n";
        std::exit(1);
    }
}

int main() {
    static_assert(sizeof(PixelColor) == 4, "RGBA ABI must be four bytes");
    shutdown_image(); shutdown_image();
    require(get_image_ptr() == nullptr && get_image_width() == 0 && get_image_height() == 0);
    update_image(0); animate_sphere_simple();
    initialize_image();
    require(get_image_ptr() && get_image_width() == 1080 && get_image_height() == 607);
    shutdown_image();
    require(initialize_image_with_options(32, 18, 2, 42) == 1);
    const PixelColor* pointer = get_image_ptr();
    require(pointer && get_image_width() == 32 && get_image_height() == 18);
    require(!initialize_image_with_options(1, 18, 2, 42));
    require(!initialize_image_with_options(32, 18, 0, 42));
    require(!initialize_image_with_options(4096, 4096, 2, 42));
    require(get_image_ptr() == pointer);
    for (int frame = 0; frame < 12; ++frame) {
        update_image(frame);
        require(get_image_ptr() == pointer);
        for (int i = 0; i < 32 * 18; ++i) require(pointer[i].a == 255);
    }
    const auto first = snapshot();
    // Never dereference the old pointer after reinitialization.
    require(initialize_image_with_options(32, 18, 2, 42));
    for (int frame = 0; frame < 12; ++frame) update_image(frame);
    require(first == snapshot());
    animate_sphere_at(NAN); // Invalid times are ignored.
    animate_sphere_at(1.0); update_image(12);
    require(initialize_image_with_options(8, 6, 1, 7));
    require(get_image_width() == 8 && get_image_height() == 6);
    update_image(0);
    shutdown_image();
    require(get_image_ptr() == nullptr && get_image_width() == 0 && get_image_height() == 0);
    update_image(1); animate_sphere_at(1.0); shutdown_image();
    std::cout << "FFI lifecycle, pointer stability, RGBA, reinitialization and seeded replay passed\n";
}
