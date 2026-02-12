# profile-rs

A basic cli tool to manage (configuration) files based on profiles.

It works by keeping a list of managed files. Adding a file to a profile means creating two copies of it. One original, and one specific to the profile.
When activating a profile, all files managed by it will be replaced by the profile-specific one.
When deactivating all managed files of all profiles will be replaced by their previously created original versions.

Before any activation, adding or removal all profiles will be activated to assure a defined state.

All file-paths will automatically be made absolute. 


### build against older libc

```shell
docker pull --platform linux/amd64 debian:11
docker run -it debian:11 /bin/bash
root@0fd864e589bb:/# apt update
root@0fd864e589bb:/# apt install git build-essential curl
```

```shell
user@0fd864e589bb:~$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/DerOrfa/profile-rs.git
cd profile-rs/
cargo build --release
```