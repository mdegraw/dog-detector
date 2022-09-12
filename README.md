# Dog Detector

## Setup
* We pipe the IP Camera stream into a fake virtual camera created with `v4l2loopback-dkms`.
This is a hack since `NetworkCamera` is not currently working in `nokhwa`.

1. Create fake camera `sudo modprobe v4l2loopback card_label="Fake Camera" exclusive_caps=1` 
2. ```$ ffmpeg -i http://<username>:<password>@<ip-address>:<port>/ -c copy -map 0 -framerate 30 -input_format mjpeg -f v4l2 /dev/video2```
3. Remove fake camera kernal module after using it `sudo modprobe --remove v4l2loopback`

* Need OpenCV installed to use `NetworkCamera`
* Linux needs [TensorFlow for C](https://www.tensorflow.org/install/lang_c)
** For Ubuntu/Debian follow instructions running `sudo ldconfig /usr/local/lib`
** For Arch Linux `cp ibtensorflow-cpu-linux-x86_64-{}-{}-{}` to `/lib/`