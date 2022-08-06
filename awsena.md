
# good intro to dpdk

## official aws

https://github.com/amzn/amzn-drivers/tree/master/userspace/dpdk

## ena poll mode driver - official dpdk

https://doc.dpdk.org/guides/nics/ena.html#overview

# prepare the kernel and dev tools

```sh
sudo dnf update -y
sudo dnf install -y kernel

# reboot and reconnect
sudo reboot

# after reboot continue 
sudo dnf groupinstall "Development Tools" -y
sudo dnf install -y git kernel-devel kernel-headers
```


check your ENA availability

`modinfo ena`

create 2nd network interface for DPDK using action -> networking and attach. if not exist create new one

check eth

`ifconfig`

you also need to install ninja-build as it will needed for global installation that required sudo

```sh
sudo su

# for al2022 please install python first
sudo dnf install python pip

pip3 install meson ninja pyelftools

export PATH=$PATH:/usr/local/bin
```

# setup igb_uio

https://github.com/amzn/amzn-drivers/tree/master/userspace/dpdk#42-igb_uio-setup

```sh
cd
git clone git://dpdk.org/dpdk-kmods
cd dpdk-kmods/linux/igb_uio
make
```

```sh
# igb_uio depends on the generic uio module
sudo modprobe uio

# Load igb_uio with Write Combining activated
sudo insmod ./igb_uio.ko wc_activate=1
```


# checkout dpdk code

```sh
cd
git clone git://dpdk.org/dpdk-stable
cd dpdk-stable
git checkout v20.11.3

cd
git clone https://github.com/amzn/amzn-drivers.git
cd amzn-drivers/userspace/dpdk
./backports/apply-patches.sh -d /home/ec2-user/dpdk-stable
```

# build dpdk

```sh
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
```

# prepare environment for DPDK

```sh
echo 4096 | sudo tee /proc/sys/vm/nr_hugepages
```

# create 2nd network interface for DPDK 

using action -> networking and attach. if not exist create new one

check eth for new network interface

`ifconfig`

# bind dpdk

```sh
# Disable the interface
sudo ifconfig [ethx|ensx] down

# check status
sudo usertools/dpdk-devbind.py --status
```
use ethx (al2) / ensx (al2022)

```
# bind
sudo usertools/dpdk-devbind.py --bind=igb_uio 00:06.0
```

# build example

ref: https://doc.dpdk.org/guides/sample_app_ug/compiling.html

```sh
sudo su
cd dpdk-stable/build

export PATH=$PATH:/usr/local/bin

meson configure -Dexamples=helloworld

ninja
cd examples
./dpdk-helloworld
```

read this first

https://doc.dpdk.org/guides/linux_gsg/index.html

if you want to build all examples

```sh
cd build
meson configure -Dexamples=all
ninja
```

if you want to build single sample with make

```sh
export PKG_CONFIG_PATH=/usr/local/lib64/pkgconfig

cd examples/helloworld
make
```

another sample to try 

https://github.com/NEOAdvancedTechnology/MinimalDPDKExamples/blob/master/DPDK_EC2_Tutorial.md#dpdk-on-aws-ec2-tutorial

ENA doesn't support promiscuous. so we should comments in port init samples

https://github.com/amzn/amzn-drivers/issues/172

ec2 instance network (VPC) is an L3 network, and all the packets a given ENI/ENA would receive will have the single dst MAC associated with the ENI/ENA

DPDK reduces the overhead of packet processing inside the operating system, which provides applications with more control of network resources such 
as ring buffers, memory, and poll-mode drivers. Combining DPDK and enhanced networking provides higher packets per second, less latency, less jitter, and more control over packet queueing. 

This combination is most common in packet processing devices that are highly impacted by networking performance such as firewalls, real-time communication processing, HPC, and network appliances.

There are additional operating system-specific enhancements such as TCP settings, driver settings, and Non-Uniform Mapping Access (NUMA) that can further increase performance.

check MTU

ifconfig | grep mtu
ip link show dev ens5

Detailed monitoring and proactive routing control can also mitigate network congestions and challenges. For highly sensitive media with multiple potential network paths, you can configure monitoring probes on Amazon EC2 instances to report on link health. That information can be used centrally to modify routes to alternative network paths that are healthy. 

Some media applications support buffering traffic before the media is played to the user. This buffering can help guard against jitter and varying network latencies. For media streams that can buffer audio or video, decreasing jitter is more important than reducing the average latency. 

There may also be additional optimizations at the operating system level to tune window scaling, interrupts, and Direct Memory Access (DMA) channels. With high-latency connections, you can try tuning different TCP parameters, such as TCP implementation, congestion window sizes, and timers.

With the data from your network tests, you can identify network bottlenecks using Amazon CloudWatch reports and operating system statistics. This is 
where trying different approaches such as placement groups, larger instance sizes, and more distributed systems increases application performance. There are tools like Bees with Machine Guns that can run a distributed load test.

One helpful tool to investigate further is a packet capture on the network. A packet capture is a raw dump of the traffi c sent or received over the network. The easiest way to do this in a VPC is to run a packet capture locally on the instance with tools such as tcpdump. 

External tools can run packet loss analysis, or you can look for TCP retransmissions that indicate packet loss. These actions are also an effective way to determine if the network has latency or if it is the application. The packet timing can determine when the host receives network packets and how quickly the application responds. 

For example, there is implicit routing for all subnets by default within a Virtual Private Cloud (VPC). This can rule out Layer 2 and Layer 3 communication issues within a VPC; thus, troubleshooting should start at Layer 4. In another example, when custom routing is set up through Amazon Elastic Compute Cloud (Amazon EC2 instances, Layer 3 troubleshooting may be required to ensure that routing is occurring as expected.


# check network througput

https://aws.amazon.com/premiumsupport/knowledge-center/network-throughput-benchmark-linux-ec2/

packet capture (tcpdump, tshark)

ping, traceroute