This detects files that are byte for byte identical. Useful for freeing up space if you have a badly organized pile of media files.

This should use multiple cores and tries to optimize for speed by using multiple passes. Accessing filesystem metadata is cheap, so first it traverses the filesystem to find files with the same size. If it finds multiple files with the same filesize, then it hashes the contents of the file. The CLI version should run on windows, linux and macos. The GUI version has been tested with windows and linux - not sure if it works on macos.

To create an ARM compatible binary, run
```sh
cross build --bin cli --release --target armv7-unknown-linux-gnueabihf
```
