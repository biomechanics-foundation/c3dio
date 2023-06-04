# c3dio

A c3d parser, writer and editor written in Rust and **available in 20+ languages**.

## Usage

Load a c3d file:

```
use c3dio::{C3d, C3dParseError};
let c3d_data = C3d::load("test.c3d");
```

Load only the parameters/header (no data):

```
use c3dio::{C3d, C3dParseError};
let c3d_parameters = C3d::parameters("test.c3d");
```

## Contributing

PRs, feature requests, and issues are welcome!

## Support

c3dio is part of the [biomech.dev](https://biomech.dev) family of open-source libraries. Consider supporting our work to help us contribute more to the body of biomechanics software.
