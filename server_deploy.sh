#!/bin/bash
cd /home/etomicbomb/RustProjects/ethan_ws/
/home/etomicbomb/.cargo/bin/cargo clean
ssh pi@192.168.0.69 "sudo rm -r /home/pi/Desktop/ethan_ws"
scp -r /home/etomicbomb/RustProjects/ethan_ws pi@192.168.0.69:/home/pi/Desktop/ethan_ws
ssh pi@192.168.0.69 "cd /home/pi/Desktop/ethan_ws && /home/pi/.cargo/bin/cargo build && sudo shutdown -r now"