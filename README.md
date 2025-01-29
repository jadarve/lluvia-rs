# rs-media-streaming

## Generate sample data

### TS

```bash
gst-launch-1.0 -v videotestsrc is-live=true ! videoconvert ! x264enc ! h264parse ! video/x-h264,stream-format=byte-stream ! mpegtsmux ! filesink location=sample.ts
```


## Dev environment

```bash
sudo apt install -y \
    ffmpeg \
    gstreamer1.0-alsa \
    gstreamer1.0-gl \
    gstreamer1.0-gtk3 \
    gstreamer1.0-libav \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-pulseaudio \
    gstreamer1.0-qt5 \
    gstreamer1.0-tools \
    gstreamer1.0-x \
    libgstreamer-plugins-bad1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer1.0-dev
```
