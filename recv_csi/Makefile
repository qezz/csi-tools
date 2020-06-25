build:
	cp -r ../csi-types ./
	cross build --release --target  mips-unknown-linux-musl
	rm -r ./csi-types

deploy: build
	scp ./target/mips-unknown-linux-musl/release/recv_csi root@192.168.2.1:/mnt/sda1/

