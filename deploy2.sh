cd /home/etomicbomb/RustProjects/ethan_ws/ &&

# deploy and run server executable
cargo +nightly build --release --target armv7-unknown-linux-gnueabihf &&
ssh -p 49431 pi@23.241.217.176 "sudo /home/pi/Desktop/server/try_kill.sh" &&

scp -P 49431 /home/etomicbomb/RustProjects/ethan_ws/target/armv7-unknown-linux-gnueabihf/release/server pi@23.241.217.176:/home/pi/Desktop/server/server &&

# deploy resources file
ssh -p 49431 pi@23.241.217.176 "sudo rm -r /home/pi/Desktop/server/resources" &&
scp -P 49431 -r /home/etomicbomb/RustProjects/ethan_ws/server/resources pi@23.241.217.176:/home/pi/Desktop/server/resources &&

#ssh -p 49431 pi@23.241.217.176 "/home/pi/Desktop/server/server"
ssh -p 49431 pi@23.241.217.176 "nohup sudo /home/pi/Desktop/server/server > /dev/null &"