#include <iostream>
#include "raylib.h"
#include "raytracer_ffi.h"

#ifndef UNCOMPRESSED_R8G8B8A8
#define UNCOMPRESSED_R8G8B8A8 7
#endif

int main() {
    std::cout << "Rendering Frames" << std::endl;
    initialize_image();

    const int SCALE = 2;
    int imageWidth = get_image_width();
    int imageHeight = get_image_height();
    int highResWidth = imageWidth * SCALE;
    int highResHeight = imageHeight * SCALE;

    const PixelColor* pixels = get_image_ptr();
    if (pixels == nullptr) {
        std::cerr << "Error loading image!" << std::endl;
        return 1;
    }

    Image imageData = { (void*)pixels, imageWidth, imageHeight, 1, UNCOMPRESSED_R8G8B8A8 };
    
    InitWindow(highResWidth, highResHeight, "Bradleys Raytracer");
    SetTargetFPS(60);

    // Create texture from the low-res image
    Texture2D lowResTexture = LoadTextureFromImage(imageData);

    // (Optional) Create a high-res texture if needed
    Image tempImage = ImageCopy(imageData);
    ImageResize(&tempImage, highResWidth, highResHeight);
    Texture2D highResTexture = LoadTextureFromImage(tempImage);
    UnloadImage(tempImage);  // Unload now that highResTexture is created

    std::cout << "Width: " << imageWidth << " Height: " << imageHeight << std::endl;

    int frameCount = 0;
    while (!WindowShouldClose()) {
        animate_sphere_simple();

        // Update image from Rust FFI
        update_image(frameCount);
        const PixelColor* lowResPixels = get_image_ptr();
        Image lowResImage = { (void*)lowResPixels, imageWidth, imageHeight, 1, UNCOMPRESSED_R8G8B8A8 };

        // Update texture with new pixel data (using the correct pointer)
        UpdateTexture(lowResTexture, lowResImage.data);
        SetTextureFilter(lowResTexture, TEXTURE_FILTER_BILINEAR);

        BeginDrawing();
            ClearBackground(RAYWHITE);
            DrawTextureEx(lowResTexture, (Vector2){0, 0}, 0.0f, SCALE, WHITE);
        EndDrawing();
        
        frameCount++;
        // Do not unload lowResTexture or lowResImage here since lowResImage is stack allocated
    }

    // Cleanup resources
    UnloadTexture(lowResTexture);
    UnloadTexture(highResTexture);
    CloseWindow();
    return 0;
}
