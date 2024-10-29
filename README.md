# Vello with Jetpack Compose

This repository demonstrates an integration between Jetpack compose and [Vello](https://github.com/linebender/vello).

## Components

This has two main components:

- `vello`, which is a Jetpack compose library, exposing control over Vello scenes.
- `app`, which is the app which uses this library.

## Getting started

Open this project in Android Studio, and run on your device or in the emulator.

## Limitations

Cleanup is not yet implemented (e.g. rotating the device will probably crash).
The library is not intended for wide use, and is only currently developed as an MVP.
I do not have experience working in Jetpack compose, so architectural decisions may be suspect.

- There is bad tearing in the emulator. This is not seen on-device.
- The technical choice to use a `SurfaceView` for each Surface.
- The only available scene is a variable font, specifically Roboto Flex.
  Additionally, only a subset of this is supported, for repository size reasons.
- System font discovery didn't work for unknown reasons.
- Performance is reasonable, but doesn't consistently hit 90fps on my device.

## License

Licensed under the Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

The files in `vello/src/main/rust/roboto_flex/` are licensed solely as described in that folder.
For clarity, these files are not licensed under the Apache 2.0 license.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.
