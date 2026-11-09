# This file works from any working directory: make -f /path/to/src/makefile all.
SOURCE_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
CRATE_DIR := $(abspath $(SOURCE_DIR)/..)
BUILD_DIR := $(CRATE_DIR)/target/release
CXX ?= g++
CXXFLAGS ?= -O2 -std=c++17 -Wall -Wextra
CARGO ?= cargo
PKG_CONFIG ?= pkg-config
PYTHON ?= python3
RAYLIB_CFLAGS = $(shell $(PKG_CONFIG) --cflags raylib)
RAYLIB_LIBS = $(shell $(PKG_CONFIG) --libs raylib)
RUST_SOURCES := $(wildcard $(SOURCE_DIR)/*.rs)

.PHONY: all rust cpp run test test-cli clean
all: cpp
rust: $(BUILD_DIR)/libraytracer.so
$(BUILD_DIR)/libraytracer.so: $(RUST_SOURCES) $(CRATE_DIR)/Cargo.toml $(CRATE_DIR)/Cargo.lock
	$(CARGO) build --locked --release --manifest-path $(CRATE_DIR)/Cargo.toml

cpp: $(BUILD_DIR)/raytracer-window
$(BUILD_DIR)/raytracer-window: $(SOURCE_DIR)/main.cpp $(SOURCE_DIR)/raytracer_ffi.h $(BUILD_DIR)/libraytracer.so
	$(PKG_CONFIG) --exists raylib
	$(CXX) $(CXXFLAGS) $(RAYLIB_CFLAGS) $(SOURCE_DIR)/main.cpp -o $@ -L$(BUILD_DIR) -Wl,-rpath,'$$ORIGIN' -lraytracer $(RAYLIB_LIBS)

$(BUILD_DIR)/ffi-smoke: $(CRATE_DIR)/tests/ffi_smoke.cpp $(SOURCE_DIR)/raytracer_ffi.h $(BUILD_DIR)/libraytracer.so
	$(CXX) $(CXXFLAGS) -I$(SOURCE_DIR) $< -o $@ -L$(BUILD_DIR) -Wl,-rpath,'$$ORIGIN' -lraytracer

test: $(BUILD_DIR)/ffi-smoke
	$(CARGO) test --locked --manifest-path $(CRATE_DIR)/Cargo.toml
	$(BUILD_DIR)/ffi-smoke
test-cli: cpp
	$(PYTHON) $(CRATE_DIR)/tests/test_cli.py

run: all
	$(BUILD_DIR)/raytracer-window $(ARGS)
clean:
	$(CARGO) clean --manifest-path $(CRATE_DIR)/Cargo.toml
