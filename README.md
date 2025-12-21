# PBR Texture Viewer

A utility application for visualizing PBR (Physically Based Rendering) texture sets, built with **Bevy** game engine and **egui** UI.

## Features

### Texture Loading
- Load complete PBR texture sets from a folder
- Automatic detection of texture types by filename patterns:
  - **Base Color**: `basecolor`, `albedo`, `diffuse`, `color`
  - **Normal**: `normal`, `normalmap`, `nrm`
  - **Roughness**: `roughness`, `rough`, `rgh`
  - **Metallic**: `metallic`, `metal`, `met`
  - **ORM**: `orm`, `arm`, `rma` (combined AO/Roughness/Metallic)
  - **Ambient Occlusion**: `ao`, `occlusion`
  - **Emissive**: `emissive`, `emission`, `glow`
  - **Height/Displacement**: `height`, `displacement`, `bump`, `depth`
- Supports PNG, JPG, TGA, BMP, TIFF formats
- Drag & drop support

### Mesh Primitives
- **Sphere**: UV sphere with adjustable tessellation
- **Plane**: Flat plane for surface materials
- **Rounded Rectangle**: Rounded corner rectangle mesh

### Material Controls
- Base color tint (RGB color picker)
- Metallic (0-1)
- Roughness (0-1)
- Normal strength (0-2x)
- Ambient Occlusion strength (0-2x)
- Emissive strength (0-5x)
- Parallax depth (0-0.2) - for height map displacement

### View Modes
- **Lit**: Full PBR shaded view
- **Base Color**: Albedo/diffuse only
- **Normals**: Normal map visualization
- **Roughness**: Roughness channel
- **Metallic**: Metallic channel
- **AO**: Ambient occlusion channel
- **Emissive**: Emissive channel
- **Height**: Height/displacement map

### Camera & Lighting
- Orbit camera (right-click drag)
- Model rotation (left-click drag)
- Scroll wheel zoom
- Adjustable light direction (yaw/pitch)
- Light color and intensity
- Ambient light intensity

### Displacement
- Parallax mapping using height maps
- Relief parallax mapping for enhanced depth
- Adjustable parallax depth

## Controls

| Input | Action |
|-------|--------|
| Left Mouse Drag | Rotate model |
| Right Mouse Drag | Orbit camera |
| Scroll Wheel | Zoom in/out |
| Drag & Drop | Load texture folder |

## Building

### Prerequisites
- Rust 1.75 or later
- A GPU with Vulkan, Metal, or DX12 support

### Build Commands

```bash
# Debug build (faster compilation with dynamic linking)
cargo build

# Release build (optimized, slower to compile)
cargo build --release

# Run
cargo run

# Run release version
cargo run --release
```

## Usage

1. Launch the application
2. Click "Open Folder" or drag a texture file onto the window
3. Select the texture folder containing your PBR textures
4. Choose a mesh primitive (Sphere, Plane, or Rounded Rect)
5. Adjust material parameters using the sliders
6. Use view modes to inspect individual texture channels

## Texture Folder Structure

Your texture folder should contain images with recognizable naming patterns:

```
my_material/
├── my_material_basecolor.png
├── my_material_normal.png
├── my_material_roughness.png
├── my_material_metallic.png
├── my_material_ao.png
├── my_material_emissive.png
└── my_material_height.png
```

Or with combined ORM texture:

```
my_material/
├── my_material_basecolor.png
├── my_material_normal.png
├── my_material_orm.png          # R=AO, G=Roughness, B=Metallic
└── my_material_height.png
```

## Performance Notes

- The debug build uses dynamic linking for faster compilation
- First compile may take a few minutes as Bevy and dependencies are built
- Subsequent compiles are much faster due to incremental compilation
- For best runtime performance, use `cargo run --release`

## License

MIT License - See LICENSE file for details.
