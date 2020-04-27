cd /home/etomicbomb/RustProjects/ethan_ws/ &&
cargo build --target armv7-unknown-linux-gnueabihf &&
ssh pi@192.168.0.69 "sudo /home/pi/Desktop/server/try_kill.sh" &&
scp /home/etomicbomb/RustProjects/ethan_ws/target/armv7-unknown-linux-gnueabihf/debug/ethan_ws pi@192.168.0.69:/home/pi/Desktop/server/ethan_ws &&
scp -r /home/etomicbomb/RustProjects/ethan_ws/resources pi@192.168.0.69:/home/pi/Desktop/server/resources &&
ssh pi@192.168.0.69 "chmod +x /home/pi/Desktop/server/ethan_ws"
#ssh pi@192.168.0.69 "nohup sudo /home/pi/Desktop/server/ethan_ws > /dev/null &"
