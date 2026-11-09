#ifndef RAYTRACER_FFI_H
#define RAYTRACER_FFI_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

typedef struct PixelColor {
    unsigned char r, g, b, a;
} PixelColor;

/* Single caller only. The Rust-owned pointer remains stable during updates,
   but becomes invalid on successful reinitialization or shutdown. Never free it
   or read it concurrently with an update. */
void initialize_image(void);
/* Returns 1 on success, 0 on invalid dimensions/samples (old state preserved).
   Dimensions >=2, <=4096, total pixels <=4194304, samples in 1..1024. */
int initialize_image_with_options(int width, int height, int samples, uint64_t seed);
void update_image(int frame);
void animate_sphere_simple(void);
void animate_sphere_at(double seconds);
int get_image_width(void);
int get_image_height(void);
const PixelColor* get_image_ptr(void);
/* Idempotent. Before initialization/after shutdown: pointer NULL, dimensions 0;
   updates and animation are no-ops. */
void shutdown_image(void);
#ifdef __cplusplus
}
#endif
#endif
