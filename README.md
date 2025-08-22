# Pocket-Flow-Meter

TODO: Fill out this small description part

## Features
- [ ] Communicate with Sensirion flow sensor
    - [ ] Work with different/any sensor (dynamically get the sensor ID and configuration stuff)
    - [ ] Get real-time data of the sensor (~10Hz ?)
    - [ ] Allow user to change acquisition parameters (R/W access to sensor register)
- [ ] Display information
    - [ ] Plot (communication) status (text)
    - [ ] Plot flow curve (of last ~30sec ?)
    - [ ] Add heartbeat/still-alive animation ?
- [ ] Communication with PC
    - [ ] Some kind of webinterface to interact more deeply with the device ?
    - [ ] over Wifi ?
    - [ ] over the USB cable ?
- [ ] Make this a nice physical device
    - [ ] nice 3d printed housing
    - [ ] Easyl to assemble

## Technical Roadmap
- [x] Compile Rust to the specific uC
- [ ] From Rust, use/control the display
- [ ] Communicate with the sensirion sensor over SPI
- [ ] Test communication through the USB cable ?
- [ ] Test communication through WiFi

See [the technical notes](./docs/technical_notes.md) for more information about this topic.

## Doc links
### Lilygo microcontroller
- https://github.com/Xinyuan-LilyGO/T-Display-S3
- https://lilygo.cc/en-pl/products/t-display-s3?variant=45396274512053

### Sensor

