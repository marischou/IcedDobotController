# IcedDobotController
Simple Application to control Dobot in Iced

## What is this?
An Iced GUI program I made for my graduation research project.
It is used to control a Dobot Magician robot arm.
It is not intended for general use. You can try to edit it for your own purpose,
but I do not guranatee plug-n-play functionality.

## Basic usage
Needs to be connected to a Dobot Magician for full fuunctionality.
Works on wayland, but might need some dependencies depending on your distribution.
Not tested on Windows at all, most likely needs a different serial communication implementation.
```
git clone https://github.com/marischou/IcedDobotController.git
cd IcedDobotController
cargo run
```


## Credits
1. jerry73204, myself: dobot in rust implementation; https://github.com/marischou/dobot-rust-fx24.git
2. airstrike: dragking dragging library: https://github.com/airstrike/dragking.git
