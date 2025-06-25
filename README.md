# Lluvia-rs

Base dependencies for installing Python 3.12 from pyenv

```bash
sudo apt install -y \
    libbz2-dev \
    libctypes-ocaml-dev \
    libcurses-ocaml-dev \
    libffi-dev \
    liblzma-dev \
    libreadline-dev \
    libsqlite3-dev \
    tk-dev
```

Python config

```bash
pyenv virtualenv 3.12.9 lluvia-rs
pyenv activate lluvia-rs
pip install maturin
```

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

## WASM

Wasm-pack: https://rustwasm.github.io/wasm-pack/installer/

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

```
wasm-pack build --out-dir '../../pkg' crates/lluvia_gpu
```

## References

* https://stackoverflow.com/questions/78211399/how-can-i-separate-a-rust-library-and-the-pyo3-exported-python-extensions-which
* https://pypi.org/project/setuptools-rust/
