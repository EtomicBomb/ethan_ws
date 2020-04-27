#!/bin/bash
ssh pi@192.168.0.69 "sudo rm -r /home/pi/Desktop/ethan_ws/resources"
scp -r /home/etomicbomb/RustProjects/ethan_ws/resources pi@192.168.0.69:/home/pi/Desktop/ethan_ws/resources