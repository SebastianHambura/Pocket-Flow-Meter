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

### Rust workspace organisation
esp32 -> target esp32
sensition-SLF -> target host
Different targets: needs some nighlty cargo features: https://users.rust-lang.org/t/can-i-configure-rust-analyzer-vscode-to-use-a-different-target-for-different-crates-in-my-workspce/123661/2
```
rustup toolchain install nightly
rustup override set nightly
```
### Application Note
Application Note
Standalone Flow Measurement Device Using Sensirion Flow Sensor and LilyGO Display Module
________________________________________
1. Overview
This application note describes the design and implementation of a standalone flow measurement device based on Sensirion flow sensors and a LilyGO microcontroller module with integrated LCD display.
The system is designed to:

•	Measure fluid flow in real time (Calibrated for H2O and IPA)

•	Display flow rate locally on an embedded screen

•	Operate independently without requiring a PC or external interface

•	Be fully reproducible using off-the-shelf components

•	Use two different flow sensors for up to ±40 ml/min (SLF3S-1300F) or ±2000 μl/min (SLF3S-0600F). The code allows automatic selection of sensor model when connected and switching of flow rate units.

•	USB-C for PC connection and power. Molex Picoblade 4 pin self-assembly (Qwiic) cable for connecting LCD and flow sensor.

This project is released as open-source hardware and software, enabling users to build, modify, and extend the system.


3. System Architecture
   
2.1 Functional Blocks
   
The system consists of three main components:

•	Flow Sensor (Sensirion SLF3S-1300F or SLF3S-0600F)

o	Provides calibrated digital flow measurement

o	Communicates via I²C interface (via Qwiic port)

•	Microcontroller (T-Display-S3 dev board with control chip ESP32-S3)

o	Reads sensor data

o	Processes and formats measurements

o	Drives the display

•	Display (Integrated LCD- 1.9" diagonal, Full-color TFT Display)

o	Shows real-time flow rate and system status

•	Cable

o	Generic USB-C

o	Cable assembly Molex PicoBlade 1.25mm for I2C Qwiic 

 
2.2 Block Diagram
 
6. Hardware Design
3.1 Bill of Materials (BOM)
   
Component	Description	Cost

Sensirion Flow Sensor	SLF3S-1300F & SLF3S-0600F	Approx. 140 Euro 

LilyGO Board	ESP32 with integrated LCD	Approx. 20 Euro

Power Supply	USB-C	

Connector	Molex PicoBlade 4-pin 1 mm for Qwiic port in the T-display side and Molex PicoBlade 1.25 mm 6-pin for the sensor side (Self assembled)	Approx. 2 Euro

3.2 Electrical Connections

Sensor Pin	MCU Pin (Example)	Description

VDD	3.3V	Power supply

GND	GND	Ground

SDA	GPIO21	I²C Data

SCL	GPIO22	I²C Clock

  
3.3 Power Considerations

•	Operates at 3.3V logic level

•	Typical consumption:

o	ESP32: ~80–240 mA 

o	Sensor: ~5–20 mA

________________________________________
4. Firmware Design
   
4.1 Development Environment

•	Platform: Rust, VStudio, ESP HALL
•	Required libraries:
o	I²C communication (Wire)
o	Display driver (TFT_eSPI or similar)
o	Sensirion sensor library
________________________________________
4.2 Functional Flow
1.	Initialize hardware (I²C, display)
2.	Detect sensor presence
3.	Periodically read flow data
4.	Convert raw data to physical units
5.	Update display
6.	Handle errors (sensor disconnect, invalid readings)
________________________________________
5. Sensor Integration
Sensor codes

SLF3S-1300F	0x07030202

SLF3S-0600F	0x07030302

SLF3C-1300F	0x07030402

SLF3S-4000B	0x07030501


5.1 Communication Protocol
•	Interface: I²C
•	Display side: Qwiic 4 pin port to 6 pin connector (2-pins are left out) on sensor side
________________________________________
5.2 Calibration
•	Sensirion sensors are typically factory calibrated
•	No additional calibration required for standard use
 
6. Assembly Instructions
1.	Assemble cable using Molex Picoblade kit following the pin configuration shown in Sensor Pin	MCU Pin (Example)	Description
2.	Connect sensor and T-Display-S3 using the self made cable
3.	Power device via USB-C (Ideally from the PC used for firmware flashing)
4.	Flash firmware using Arduino IDE / PlatformIO
5.	Verify sensor model and unit on the display output 
6.	Check button functions (Start/Stop & H2O/IPA)
7.	Connect flow source and verify flow rate using a controlled source (e.g. a syringe pump)
________________________________________
7. Future Improvements
   
•	Data logging (SD card)

•	Wireless connectivity (BLE/WiFi)

•	Web dashboard

•	Built-in Battery

•	Multi-sensor support

________________________________________
8. Conclusion
If you are still reading this document, you are awesome. Hope this helps. If you want to buy us a coffee, paypal me at neloy.sadat@gmail.com.
If you have questions, use the same email address. I cant guarantee an answer.
________________________________________
9. References
•	Sensirion datasheets (specific sensor model)
•	LilyGO board documentation
•	GitHub link
10. Licensing
•	Open-source licensing. The code is available for future modifications.

