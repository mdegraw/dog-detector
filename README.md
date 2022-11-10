# Dog Detector

### Usage
* Create config file in `~/.config/dog-detector/config.toml` with path to tensorflow model file and settings

## Setup
* We pipe the IP Camera stream into a fake virtual camera created with `v4l2loopback-dkms`.
This is a hack since `NetworkCamera` is not currently working in `nokhwa`.

1. Create fake camera `sudo modprobe v4l2loopback card_label="Fake Camera" exclusive_caps=1` 
2. ```$ ffmpeg -i http://<username>:<password>@<ip-address>:<port>/ -c copy -map 0 -framerate 30 -input_format mjpeg -f v4l2 /dev/video2```
3. Remove fake camera kernal module after using it `sudo modprobe --remove v4l2loopback`

* Linux dependencies
** `libssl-dev`
** `v4l-utils`

* Linux needs [TensorFlow for C](https://www.tensorflow.org/install/lang_c)
** Follow instructions running `sudo ldconfig /usr/local/lib`

* Optional - need OpenCV installed to use `NetworkCamera`

### Raspberry Pi
* Currently using a USB Webcam instead of a PiCam due to issues on an ARM64 OS
* Install dependencies
1. sudo apt install v4l-utils
2. Follow instructions [here](https://qengineering.eu/install-tensorflow-on-raspberry-64-os.html) for installing the `TensorFlow 2.8.0 C++ API`

### Building the Docker Images
* For building the base arm64 image
** `docker buildx build --load --platform linux/arm64 --file dockerfiles/rust-tf-arm64-base.Dockerfile -t rust-tensorflow-arm64 .`
* For building the arm64 release image
** `docker buildx build --load --platform linux/arm64 --file dockerfiles/arm64.Dockerfile -t arm64-rust-build-release .`
* Building an arm64 release
** `docker run --platform linux/arm64 --rm -v <PATH TO APPLICATION>:/app arm64-rust-build-release`
