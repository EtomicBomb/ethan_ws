cd /home/etomicbomb/RustProjects/ethan_ws/ &&

# deploy resources file
ssh pi@192.168.0.69 "sudo rm -r /home/pi/Desktop/server/resources" &&
scp -r /home/etomicbomb/RustProjects/ethan_ws/server/resources pi@192.168.0.69:/home/pi/Desktop/server/resources &&

# deploy and run server executable
cargo build --release --target armv7-unknown-linux-gnueabihf &&
ssh pi@192.168.0.69 "sudo /home/pi/Desktop/server/try_kill.sh" &&
scp /home/etomicbomb/RustProjects/ethan_ws/target/armv7-unknown-linux-gnueabihf/release/server pi@192.168.0.69:/home/pi/Desktop/server/server &&
#ssh pi@192.168.0.69 "/home/pi/Desktop/server/server"
ssh pi@192.168.0.69 "nohup sudo /home/pi/Desktop/server/server > /dev/null &"