# Receive CSI

## Reqs

The most convenient way to cross-compile the app is to use `cross`:
https://github.com/rust-embedded/cross

## Compile

Cross-compilation requires `csi-types` module to be either present locally or
received from https://crates.io. `csi-types` is not released to crates,
therefore you need to copy it manually.

To cross-compile use make:

```
make
```

## Deploy

Use SCP (or any other method) to copy the app to a router

```
scp ./target/mips-unknown-linux-musl/release/recv_csi root@192.168.2.1:/mnt/sda1/
```

or

```
make deploy
```

## Run

`ssh` to the router, `cd` to the target directory, then run as following

```
./recv_csi --addr http://192.168.2.10:8899/csi
```
