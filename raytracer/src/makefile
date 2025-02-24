.PHONY: rust cpp all run clean

# Build the Rust library in release mode.
rust:
	cargo build --release

# Build the C++ binary, linking against the Rust library.
cpp: rust
	g++ main.cpp -o my_app -L../target/release -Wl,-rpath,../target/release -lraytracer -lraylib -lGL -lm -lpthread -ldl

# "all" target builds the C++ binary.
all: cpp

# Run the binary.
run: all
	./my_app

# Clean both Rust and C++ build artifacts.
clean:
	cargo clean
	rm -f my_app
