#include "raylib.h"

int main() {
    // Define screen dimensions
    const int screenWidth = 800;
    const int screenHeight = 450;
    
    // Initialize window with title "raylib Example"
    InitWindow(screenWidth, screenHeight, "raylib Example");

    // Set the target FPS (frames per second)
    SetTargetFPS(60);

    // Main game loop: run until the window is closed
    while (!WindowShouldClose()) {
        // Start drawing
        BeginDrawing();
        
        // Clear the screen to a white background
        ClearBackground(RAYWHITE);
        
        // Draw some text on the screen
        DrawText("Hello, raylib!", 190, 200, 20, LIGHTGRAY);
        
        // Finish drawing and swap buffers
        EndDrawing();
    }

    // Close the window and cleanup resources
    CloseWindow();
    return 0;
}
