"""Render the production Rust scene through the C ABI, with no window or packages."""
import argparse
import ctypes as C
import hashlib
import json
from pathlib import Path


class Pixel(C.Structure):
    _fields_ = [(channel, C.c_ubyte) for channel in 'rgba']


def render(output: Path, animated: bool):
    library = Path(__file__).resolve().parents[1] / 'target/release/libraytracer.so'
    lib = C.CDLL(str(library))
    lib.initialize_image_with_options.argtypes = [C.c_int, C.c_int, C.c_int, C.c_uint64]
    lib.initialize_image_with_options.restype = C.c_int
    lib.get_image_ptr.restype = C.POINTER(Pixel)
    lib.update_image.argtypes = [C.c_int]
    lib.update_image.restype = None
    lib.animate_sphere_at.argtypes = [C.c_double]
    lib.animate_sphere_at.restype = None
    lib.shutdown_image.argtypes = []
    lib.shutdown_image.restype = None
    width, height, samples, frames, seed = 160, 90, 5, 100, 42
    if not lib.initialize_image_with_options(width, height, samples, seed):
        raise RuntimeError('Renderer rejected demo settings')
    means = []
    try:
        for frame in range(frames):
            if animated:
                lib.animate_sphere_at(frame / 60.0)
            lib.update_image(frame)
            pixels = lib.get_image_ptr()
            assert all(pixels[i].a == 255 for i in range(width * height))
            rgb = bytes(channel for i in range(width * height) for channel in (pixels[i].r, pixels[i].g, pixels[i].b))
            means.append(sum(rgb) / len(rgb))
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(f'P6\n{width} {height}\n255\n'.encode() + rgb)
        report = dict(width=width, height=height, samples=samples, frames=frames, seed=seed,
                      animated=animated, sha256_rgb=hashlib.sha256(rgb).hexdigest(),
                      mean_rgb_frames_6_15=sum(means[5:15])/10,
                      mean_rgb_frames_91_100=sum(means[90:100])/10)
        if not animated:
            assert abs(report['mean_rgb_frames_91_100']/report['mean_rgb_frames_6_15'] - 1) < .1
        output.with_suffix('.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))
    finally:
        lib.shutdown_image()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=Path('outputs/static.ppm'))
    parser.add_argument('--animated', action='store_true')
    args = parser.parse_args()
    render(args.output, args.animated)
