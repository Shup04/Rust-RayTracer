# Rust-RayTracer

A Rust CPU ray tracer that grew from static images into an animated C++/raylib window. It explores how a small set of geometric and material rules can produce an image, then how to keep displaying that image while the scene changes. Gravity-inspired ray bending, partial-frame updates, and temporal sampling are experiments along that path.

The latest implementation connects the renderer to a live display, but its temporal accumulator has a known brightness defect. This is an unfinished rendering experiment, with useful geometry, material, and display components; it is not a finished engine or a validated physical simulation.

## The idea and how it developed

The February–March 2025 history shows a concrete progression:

1. **Construct an image from rays.** Early commits introduce vector arithmetic, a sky gradient, sphere intersections, a world containing multiple objects, cubes, reflections, and multiple samples per pixel. These pieces remain recognizable in the current modules.
2. **Make surfaces behave differently.** Diffuse scattering and fuzzy metal reflections turn the geometry into a material experiment. A ray can bounce recursively, carrying a surface's color into the result.
3. **Explore bent light paths.** The gravity commits experiment with redirecting rays near a mass rather than treating every path as one straight line. Commit messages describe black-hole renders, but the present equations should be understood as a visual experiment, not evidence of a general-relativistic solution.
4. **Move from completed images to a running window.** The later work adds an RGBA buffer, a C interface, and C++ window code. Commits progress from opening a window after rendering to displaying live frames and animated geometry.
5. **Trade rendering work against motion and noise.** Commit `cd2c672` updates one phase of a 3×3 pixel grid per frame, retaining the other pixels. The final commit, `d6a987b` (“temporal sampling almost works”), replaces that selection with new samples for every pixel and an exponential moving average across frames.

That progression explains the mixed-language layout: Rust owns the scene and ray calculations, while C++ supplies the window and texture presentation. The central unfinished question is whether repeated, inexpensive samples can produce a stable moving image. There are no measured frame-rate or image-quality results in this README.

## The active rendering path

```text
C++ window loop
  → animate the first sphere
  → request a Rust image update
      → generate camera rays for each pixel in parallel
      → intersect the scene and scatter through materials
      → average this frame's samples
      → blend with the pixel's previous color
  → read Rust's RGBA buffer
  → update and draw a raylib texture
```

[`raytracer/src/main.rs`](raytracer/src/main.rs) contains the scene, rendering constants, recursive `ray_color`, parallel image update, and exported C functions. The scene is built directly in Rust: two spheres and three boxes, including a broad ground box. There is no scene-file loader or editor.

[`camera.rs`](raytracer/src/camera.rs) constructs a fixed camera and viewport. Each pixel receives randomized sample positions. The current constants request a width of 1080, a 16:9 aspect ratio, five samples per pixel per update, and a maximum scattering depth of three. Integer conversion makes the actual buffer 1080×607. These are source constants, not command-line options.

[`hittable_list.rs`](raytracer/src/hittable_list.rs) checks the scene's objects and retains the nearest hit. [`sphere.rs`](raytracer/src/sphere.rs) solves the sphere intersection; [`cube.rs`](raytracer/src/cube.rs) intersects axis-aligned boxes. [`material.rs`](raytracer/src/material.rs) separates geometry from scattering behavior: Lambertian surfaces scatter around the normal, while metals reflect with a configurable random fuzz. A miss produces the sky gradient.

Rayon distributes pixel updates across CPU workers. Raylib/OpenGL presents the resulting texture; it does not perform this ray tracing on the GPU. The current frame loop samples **every pixel**. The earlier interleaved-grid optimization is historical, not an additional active optimization in the final commit.

## Why temporal accumulation is unfinished

[`color_accumulator.rs`](raytracer/src/color_accumulator.rs) contains both a sum-based accumulator and an exponential moving average (EMA). The renderer uses the EMA with `ALPHA = 0.5`:

```text
stored_color = alpha × new_frame_color + (1 − alpha) × stored_color
```

This keeps recent samples influential instead of weighting all past frames equally. It is a plausible mechanism for smoothing sampling noise while a scene moves, although motion-related artifacts have not been evaluated here.

**The active output conversion is wrong:** `update_ema` already stores an average, but `to_pixel_color` divides it by the accumulated frame count again. An isolated check of the original accumulator with repeated white inputs produced red-channel values of 255 on frame 1, 127 on frame 2, and 2 on frame 100. A constant input therefore darkens over time. That check establishes this arithmetic defect; it does not establish that the full application currently builds or renders correctly.

The capitalized [`ColorAccumulator.rs`](raytracer/src/ColorAccumulator.rs) is an older, separate file. The `mod color_accumulator` declaration selects the lowercase file described above. Editing the older file would not change the active renderer.

## Rust/C++ boundary and display

[`raytracer_ffi.h`](raytracer/src/raytracer_ffi.h) declares the C interface and matching four-byte RGBA pixel layout. Rust initializes and owns the image and accumulation buffers. C++ requests an update, then reads the pixel pointer to upload a texture. The pointer is not an ownership transfer to C++.

The active executable is [`raytracer/src/main.cpp`](raytracer/src/main.cpp). It initializes the buffers, opens a window at twice the render dimensions, animates the first sphere, and repeatedly updates the texture. `SetTargetFPS(60)` is a requested display rate, not a demonstrated rendering speed. Rust's `main()` is empty; `cargo run` is not the route to this window.

The exported buffer functions use mutable global state, and sphere animation assumes the first scene object is a sphere. The existing C++ loop calls them serially. A different client must not assume the raw pointer survives buffer reinitialization or that concurrent calls are supported.

## Other experiments in the tree

| Path or feature | Relationship to the current renderer |
| --- | --- |
| Gravity branch in `ray_color` | Steps the ray through short segments and bends its direction using an inverse-square acceleration. `SINGULARITY` is currently `false`, so the default scene uses straight rays. |
| [`volumetric.rs`](raytracer/src/volumetric.rs) | Medium/scattering experiment; its module and imports are commented out in the active Rust entry point. |
| [`raytracer/cpp/main.cpp`](raytracer/cpp/main.cpp) | Earlier display experiment with an older interface. The active makefile does not build it. |
| [`raytracer/cpp/vulkanTest/`](raytracer/cpp/vulkanTest/) | Separate Vulkan/GLFW application and compute shader. The shader contains sphere-intersection code, but its scene traversal and scattering loop remain placeholders. It is not a GPU backend selected by the Rust renderer. |

Finishing the CPU renderer does not require completing these separate experiments. Their code and history preserve the directions explored without making them part of the current build path.

## Build and run

The supplied build is oriented toward a Linux desktop environment. It requires:

- Rust and Cargo, for the Rust 2021 crate and shared library.
- GNU Make and `g++`.
- raylib headers and a linkable raylib library, plus the OpenGL and system libraries named by the makefile (`GL`, `m`, `pthread`, and `dl`).
- A graphical session for the window.

[`Cargo.toml`](raytracer/Cargo.toml) declares `cgmath`, `rand`, `rayon`, `rusty-ppm`, and `lazy_static`. There is no committed `Cargo.lock` in this checkout, so resolving dependencies is not a recorded reproducible environment. A working Rust/raylib toolchain combination still needs to be established.

From the repository root:

```sh
cd raytracer/src
make -f makefile all
./my_app
```

The [makefile](raytracer/src/makefile) first runs `cargo build --release`, then links `main.cpp` against the Rust library in `../target/release` and raylib. Keep that working directory when launching: the embedded library search path is relative. `make -f makefile run` performs both steps. Building replaces the checked-in `my_app`; existing binaries are not evidence that a fresh source build works on another machine.

These commands were checked against the source but were not executed while preparing this documentation. No dependencies were installed. The window has no scene-editing controls; scene, camera, sampling, and gravity changes currently require editing and rebuilding the relevant source.

## Definition of done

A bounded finish is a reproducible CPU renderer demonstration with stable output and an explicit record of the experiments left outside it. It does not require a general-purpose engine, a Vulkan port, or physically accurate black-hole rendering.

- [ ] Build the Rust library and active C++ window from source in one documented environment, record the working dependency/toolchain versions, and demonstrate startup and normal shutdown without relying on the committed binary.
- [ ] Correct the temporal accumulator's normalization and verify that repeated identical samples keep the same brightness over many updates; make the intended sum-versus-EMA behavior explicit.
- [ ] Demonstrate the existing spheres, boxes, diffuse surfaces, and fuzzy metals in a static scene, then the animated sphere. Record sampling settings and check that temporal blending does not progressively darken the image; record remaining motion/noise artifacts honestly.
- [ ] Verify the existing single-caller buffer lifecycle: initialize once, update repeatedly, read/upload the RGBA buffer, and close. Record the ownership and reinitialization limits so the display does not depend on an accidental FFI assumption.
- [ ] Preserve a representative screenshot or short capture with its settings and a short conclusion. Identify gravity, volumetrics, and Vulkan as either separately demonstrated experiments or intentionally unfinished historical work, rather than prerequisites that keep this renderer open forever.
