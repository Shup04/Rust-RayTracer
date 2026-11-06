#include <cstdlib>
#include <algorithm>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>
#include "raylib.h"
#include "raytracer_ffi.h"

static bool decimal_digits(const std::string& value) {
    return !value.empty() && std::all_of(value.begin(), value.end(), [](char c) {
        return c >= '0' && c <= '9';
    });
}

static int positive_int(const char* value) {
    std::size_t consumed = 0;
    const std::string text(value);
    if (!decimal_digits(text)) throw std::invalid_argument("expected positive decimal integer");
    int parsed = std::stoi(text, &consumed);
    if (consumed != text.size() || parsed <= 0) throw std::invalid_argument("expected positive integer");
    return parsed;
}

int main(int argc, char** argv) {
    int width = 1080, height = 607, samples = 5, frames = 0;
    bool animate = true;
    uint64_t seed = 1;
    std::string output, screenshot;
    try {
        for (int i = 1; i < argc; ++i) {
            const std::string option(argv[i]);
            if (option == "--static") { animate = false; continue; }
            if (option == "--help") {
                std::cout << "Usage: raytracer [--static] [--width N --height N] [--samples N] "
                             "[--seed N] [--frames N] [--output frame.ppm] [--screenshot window.png]\n";
                return 0;
            }
            if (++i == argc) throw std::invalid_argument("missing option value");
            const std::string value(argv[i]);
            if (value.empty() || value.rfind("--", 0) == 0)
                throw std::invalid_argument("missing option value: " + option);
            if (option == "--width") width = positive_int(argv[i]);
            else if (option == "--height") height = positive_int(argv[i]);
            else if (option == "--samples") samples = positive_int(argv[i]);
            else if (option == "--frames") frames = positive_int(argv[i]);
            else if (option == "--seed") {
                std::size_t consumed = 0;
                if (!decimal_digits(value)) throw std::invalid_argument("invalid seed: expected unsigned decimal integer");
                seed = std::stoull(value, &consumed);
                if (consumed != value.size()) throw std::invalid_argument("invalid seed");
            }
            else if (option == "--output") output = argv[i];
            else if (option == "--screenshot") screenshot = argv[i];
            else throw std::invalid_argument("unknown option: " + option);
        }
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 2;
    }
    if (!initialize_image_with_options(width, height, samples, seed)) {
        std::cerr << "Invalid render dimensions or sample budget\n";
        return 2;
    }
    InitWindow(width * 2, height * 2, "Rust-RayTracer CPU demo");
    if (!IsWindowReady()) { shutdown_image(); return 1; }
    SetTargetFPS(60);
    Image image = {const_cast<PixelColor*>(get_image_ptr()), width, height, 1, PIXELFORMAT_UNCOMPRESSED_R8G8B8A8};
    Texture2D texture = LoadTextureFromImage(image);
    if (texture.id == 0) { CloseWindow(); shutdown_image(); return 1; }
    SetTextureFilter(texture, TEXTURE_FILTER_BILINEAR);
    int frame = 0;
    while (!WindowShouldClose() && (frames == 0 || frame < frames)) {
        if (animate) animate_sphere_at(frame / 60.0);
        update_image(frame);
        UpdateTexture(texture, get_image_ptr());
        BeginDrawing();
        ClearBackground(RAYWHITE);
        DrawTextureEx(texture, Vector2{0, 0}, 0, 2, WHITE);
        EndDrawing();
        ++frame;
    }
    if (!screenshot.empty()) TakeScreenshot(screenshot.c_str());
    int result = 0;
    if (!output.empty()) {
        std::ofstream file(output, std::ios::binary);
        file << "P6\n" << width << ' ' << height << "\n255\n";
        const PixelColor* pixels = get_image_ptr();
        for (int i = 0; i < width * height; ++i) {
            const char rgb[] = {static_cast<char>(pixels[i].r), static_cast<char>(pixels[i].g), static_cast<char>(pixels[i].b)};
            file.write(rgb, 3);
        }
        file.close();  // Include buffered write/close errors in the exit status.
        if (!file) { std::cerr << "Failed to write " << output << '\n'; result = 1; }
    }
    std::cout << "Completed " << frame << " frames at " << width << 'x' << height
              << ", samples=" << samples << ", seed=" << seed << ", animated=" << animate << '\n';
    UnloadTexture(texture);
    CloseWindow();
    shutdown_image();
    return result;
}
