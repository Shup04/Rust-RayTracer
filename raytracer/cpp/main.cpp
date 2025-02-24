#include <iostream>
#include "raylib.h"
#include "raytracer_ffi.h"

// Color Struct
typedef struct PixelColor {
    unsigned char r;
    unsigned char g;
    unsigned char b;
    unsigned char a;
} PixelColor;

// Rust FFI Functions
extern "C" {
  void update_image();
  int get_image_width();
  int get_image_height();
  const PixelColor* get_image_ptr();
}

int main() {
  // Initial image computation
  update_image();

  // Load image from rust FFI functions;
  int imageWidth = get_image_width();
  int imageHeight = get_image_height();
  const PixelColor* pixels = get_image_ptr();
  if (pixels == nullptr) {
    std::cerr << "Error loading image!" << std::endl;
    return 1;
  }

  // Create an image from the vector of pixels given by rust
  // NOTE: the pixel data is stored in row major order in rgba format with 255 colors.
  Image imageData = {
    (void*)pixels, // Pointer
    imageWidth,
    imageHeight,
    1, // Mipmaps
    //UNCOMPRESSED_R8G8B8A8 // Format
  };

  // Initialize the window
  InitWindow(imageWidth, imageHeight, "Bradleys Raytracer");
  SetTargetFPS(60);

  // Create a texture from the image data
  Texture2D texture = LoadTextureFromImage(imageData);
  UnloadImage(imageData);

  while (!WindowShouldClose()) {
    BeginDrawing();
    ClearBackground(RAYWHITE);
    DrawTexture(texture, 0, 0, WHITE);
    EndDrawing();
  }

  // Close the window and cleanup resources
  UnloadTexture(texture);
  CloseWindow();
  return 0;
}
