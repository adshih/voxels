# Voxels

## Prerequisites

- [Install Rust](https://www.rust-lang.org/tools/install)
- [Bevy dependencies](https://bevyengine.org/learn/quick-start/getting-started/setup/) (platform-specific)

## Structure

- `client/` - Game client with rendering and player controls
- `voxel-net/` - Multiplayer server and networking
- `voxel-core/` - Shared voxel data structures
- `voxel-world/` - World generation and terrain logic

## Usage

**Server:**
```bash
cargo run --bin voxel-net
```

**Client (singleplayer):**
```bash
cargo run --bin client
```

**Client (multiplayer):**
```bash
cargo run --bin client -- --connect 127.0.0.1:8080 --name YourName
```
