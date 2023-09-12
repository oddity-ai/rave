<h1 align="center">
  <code>rave</code>
</h1>
<p align="center">Rust audio/video engine</p>
<div align="center">

[![version](https://img.shields.io/crates/v/ravelib)](https://crates.io/crates/ravelib)
[![license](https://img.shields.io/crates/l/ravelib)](#%EF%B8%8F-license)
[![docs](https://img.shields.io/docsrs/ravelib)](https://docs.rs/ravelib)

</div>

`rave` is a work-in-progress audio and video library with a focus on streaming applications.

## 🛠 Status

`rave` is under heavy development and not ready for use yet.

## 🎯 Goals

* Easy-to-use and safe API.
* Quality over quantity (well-behaved encoders and decoders over many codecs).
* Minimize the number of external non-Rust dependencies.

Note that `rave` is not meant to be a replacement for `ffmpeg`. Use `ffmpeg` if you need support for
many formats and codecs.

## ✨ Credits

`rave` only exists thanks to the following organizations and people:

* Everyone who worked on [video-rs](https://github.com/oddity-ai/video-rs)!
* [Cisco](https://cisco.com) for developing [openh264](https://github.com/cisco/openh264).
* [Ralf Biedert](https://github.com/ralfbiedert) for maintaining [`openh264-rs`](https://github.com/ralfbiedert/openh264-rs), a Rust wrapper for `openh264`.
* [Provincie Utrecht](https://www.provincie-utrecht.nl/) for supporting this project as part of the "Situational Awareness Software" project.
* The [FFmpeg project](https://ffmpeg.org/) for `ffmpeg` and the `ffmpeg` libraries.

## ⚖️ License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.