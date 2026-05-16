
To program Lilygo: using probe-rs ? 
Lilygo -> SoC: ESP32-S3R8

https://github.com/esp-rs/awesome-esp-rust?tab=readme-ov-file 
https://github.com/aedm/esp32-s3-rust-axum-example

Lilygo's display: compatible with ST7789
Rust driver: mipidsi should have a driver for ST7789
- mipidsi is compatible with embedded-graphics
- kolibri-embbeded-gui is compatible with embedded-graphics
- For plots: https://crates.io/crates/embedded-plots ? or maybe https://crates.io/crates/embedded-charts ?


Other interesting tutorials:
- https://github.com/mgrenonville/esp32-mipidsi-clock
- https://github.com/georgik/esp32-rust-lilygo-t5-epaper 

Simulation using Wokwi ?

Memo to myself: to generate a independent image that contains everything that you can flash on the device at adress 0x0, run :
```
cargo espflash save-image --chip esp32s3 --bin flow-meter --release firmware.bin --merge
```