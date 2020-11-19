Rust Bindings for DPDK
=======================

This crate provides Rust bindings for [DPDK](https://www.dpdk.org/). The
following drivers are supported:

- [x] Mlx4
- [x] Mlx5

## Build configuration

This crate assumes that you have a system-wide installation for DPDK, where both
`pkg-config` and `ld` may find it. If not, set `PKG_CONFIG_PATH` when building
this crate.
