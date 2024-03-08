# Embedded-gfx
<a href="https://crates.io/crates/embedded-gfx"><img alt="crates.io" src="https://img.shields.io/crates/v/embedded-gfx"></a>
<a href="https://github.com/Kezii/embedded-gfx/actions"><img alt="actions" src="https://github.com/Kezii/embedded-gfx/actions/workflows/rust.yml/badge.svg"></a>

This is an opengl-like library to draw 3D graphics in an embedded system, built around embedded-graphics.

## Features

- [x] full mvp pipeline with perspective projection
- [x] point cloud rendering
- [x] wireframe rendering
- [x] solid color triangle rendering
- [x] simple per-triangle lighting
- [x] mesh transformation
- [x] mesh loading from stl files

## Todo
- [ ] z-buffer
- [ ] per-fragment interpolation
- [ ] proper pipeline for vertex / fragment shading
- [ ] texture mapping ?

## Example

You can find a working example in the *Rust on M5Stack Cardputer* project

https://github.com/Kezii/Rust-M5Stack-Cardputer

https://github.com/Kezii/Rust-M5Stack-Cardputer/assets/3357750/658bd303-03c5-4dc2-9c49-75a98b667719
