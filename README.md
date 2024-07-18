<div align="center">

# Vent-Engine

**A game engine written in Rust with the goal to be very fast & user-friendly**

![CI](https://github.com/Snowiiii/Vent-Engine/actions/workflows/rust.yml/badge.svg)
![Apache_2.0](https://img.shields.io/badge/license-Apache_2.0-blue.svg)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/f9d502f771314c628eee53e1369c750a)](https://app.codacy.com/gh/Snowiiii/Vent-Engine/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)

</div>

### 🏆 Goals

- **Built in Rust:** This engine leverages the power of Rust and avoids external language bindings as much as possible.
- **Performance Optimization:** Vulkan is used for top-tier performance through native APIs.
- **User-Friendly Design:** The engine prioritizes ease of use.
- **Cross-Platform Support:** One goal of the engine is to support various platforms ([Platforms](#platforms)).

### 🏗 Current Status
Vent-Engine is currently in heavy development, Here is how it currently looks:
![image](https://github.com/Snowiiii/Vent-Engine/assets/71594357/5dd81844-9d01-4795-a0fc-4f9e5a5c1a4e)
**(09.07.2024)**

### How to run?
This section explains how to compile and run the Vent Engine from source code. Since it's under heavy development, there are currently no pre-built releases available.

#### Prerequisites:
- **Rust compiler**: Download and install Rust from the official website: https://www.rust-lang.org/tools/install
- **Vulkan-compatible GPU** The Vent Engine utilizes Vulkan for graphics rendering. You'll need a graphics card that supports Vulkan
#### Steps:
1. **Clone the repository:**
`git clone https://github.com/ventengine/Vent-Engine.git`
2. **Compile & Run:** 
`cargo run --bin vent-runtime`


### How to contribute?

Contributions are welcome in any way, shape, or form. See [Contributing](CONTRIBUTING.md) to know how you can get started.

### 🎮 Platforms

Vent-Engine Platform Support:

| Platform | Runtime | Editor |
| -------- | ------- | ------ |
| Windows  | 😬     | **❓** |
| MacOS    | **❓**  | **❓** |
| Linux     | ✅️     | **❓** |
| Redox    | **❓**  | **❓** |
| VR       | **❓**  | ❌     |
| Android  | **❓**  | ❌     |
| IOS      | **❓**  | ❌     |
| WASM     | **❓**  | ❌     |

- ✅: Works as intended
- ❌ Will not be Supported
- 😬: Mostly works but Unstable
- ❓: Unknown status

### 📝 License

Vent-Engine uses the [Apache 2.0 License](LICENSE)
