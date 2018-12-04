This detects files that are byte for byte identical. Useful for freeing up space if you have a badly organized pile of media files.

This should use multiple cores and tries to optimize for speed by using 3 passes. First it traverses the filesystem to find files with the same size, then it hashes the first 1000 bytes, then it hashes the whole file.
