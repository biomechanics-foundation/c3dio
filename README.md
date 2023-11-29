# c3dio

![Crates.io](https://img.shields.io/crates/v/c3dio.svg)

A c3d parser, writer and editor written in Rust.

## Usage

Load a c3d file:

```rust
use c3dio::{C3d, C3dParseError};
let c3d_data = C3d::load("test.c3d");
```

Load only the parameters/header (no data):

```rust
use c3dio::{C3d, C3dParseError};
let c3d_parameters = C3d::load_parameters("test.c3d");
```

## Contributing

PRs, feature requests, and issues are welcome!

## Support

`c3dio` is a stand-alone crate used in [Chiron](https://chiron.rs), an open-source biomechanics simulation and modeling package supported by the Biomechanics Foundation.

Consider supporting our work to help us contribute more to the body of biomechanics software.
