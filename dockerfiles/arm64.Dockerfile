FROM mdegraw001/rust-tensorflow-arm64:latest

WORKDIR /app
 
CMD ["cargo", "build", "--release"]
