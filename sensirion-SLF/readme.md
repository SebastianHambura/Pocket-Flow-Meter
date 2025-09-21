Work in progress. 

Idea for a better user interface: 
- type for flow: u16 and conversion to physical unit
- types/struct for Signal bitflag (see table 9)
- type for Product number (see table 14)
- remove command enum (or make it internal/private), and instead create a MediumType enum for Water (default) and Ethanol
    -> for the start_measurement function
- Maybe do some better Error handling 
    -> ("While no measurement data is available yet, the sensor will respond with a NACK to the I2C read header (I2C address + read bit)." )
    -> "After the reset command, the sensor will take maximum 25 ms to reset. During this time the sensor will not acknowledge its address nor accept commands."
