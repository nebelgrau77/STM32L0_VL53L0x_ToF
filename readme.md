# Time of Flight sensor

Testing STM VL53L0X time-of-flight sensor with  Rust on STM32L031 (Nucleo32 L031K6 board) 
At the moment it's just reading the distance in single mode, and printing out to serial.
If the distance is below 100 mm, the built-in LED is turned on.

TO DO:
* print the distance to an OLED display
* use a timer to trigger measurements



