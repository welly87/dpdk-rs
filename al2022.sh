#!/bin/bash

set -e

sudo dnf install -y git 

PACKAGES="kernel kernel-devel kernel-headers librdmacm libmnl-devel clang numactl-devel pkg-config python3 python3-pip meson"

sudo dnf update
sudo dnf group install "Development Tools" -y 
sudo dnf install -y $PACKAGES

sudo su

pip3 install pyelftools meson ninja

# add to environment when you want to use
export PATH=$PATH:/usr/local/bin

# setup igb_uio

cd
git clone git://dpdk.org/dpdk-kmods
cd dpdk-kmods/linux/igb_uio
make

sudo modprobe uio
sudo insmod ./igb_uio.ko wc_activate=1

# checkout dpdk with aws patch

cd
git clone git://dpdk.org/dpdk-stable
cd dpdk-stable
git checkout v20.11.3

cd
git clone https://github.com/amzn/amzn-drivers.git
cd amzn-drivers/userspace/dpdk
./backports/apply-patches.sh -d /home/ec2-user/dpdk-stable

sudo su

cd /home/ec2-user/dpdk-stable

# should load the tools like meson ninja etc
export PATH=$PATH:/usr/local/bin

meson build
cd build
ninja
ninja install

# its installed in /usr/local/lib64 /usr/local/bin

# load module

touch /etc/ld.so.conf.d/dpdk.conf

# enter /usr/local/lib64 /usr/local/lib

ldconfig

# export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/bin:/usr/local/lib64

export PKG_CONFIG_PATH=/usr/local/lib64/pkgconfig
# enter /usr/local/lib and /usr/local/lib64 /usr/local/include /usr/local/bin

# huge pages

echo 4096 | sudo tee /proc/sys/vm/nr_hugepages