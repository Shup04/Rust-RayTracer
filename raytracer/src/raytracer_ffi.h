#ifdef __cplusplus
extern "C" {
#endif

typedef struct PixelColor {
    unsigned char r;
    unsigned char g;
    unsigned char b;
    unsigned char a;
} PixelColor;

void initialize_image();
void update_image();
int get_image_width();
int get_image_height();
const PixelColor* get_image_ptr();

#ifdef __cplusplus
}
#endif
