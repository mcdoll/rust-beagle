# rust-beagle

An example implementation of a baremetal program for the beaglebone-black using rust.

## Building

The crates armv7 and sitara are available at the same github account.
To build rust-beagle set up the nightly compiler
```
    rustup override add nightly
```
Then use make to create the binary.
On the beaglebone-black you need a SD card with uboot installed and a UART connection.
Follow the instructions on how to load code to the beagle on the [osdev-wiki](https://wiki.osdev.org/ARM_Beagleboard).
The code has to be loaded to the address 0x80010000.

```
    loady 0x80010000
    go 0x80010000
```

## Acknowledgements
The code is heavily inspired by the raspberry pi tutorial by [Andre Richter](https://github.com/andre-richter).

## License
[GPLv3](LICENSE)
