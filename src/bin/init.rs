// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![cfg_attr(feature = "strict", deny(clippy:all))]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(unaligned_references)]

use dpdk_rs::{rte_eal_init, *};
use std::{env, ffi::CString, mem::MaybeUninit, time::Duration};

fn main() {
    load_mlx_driver();

    let mut args = vec![];
    let mut ptrs = vec![];
    
    for arg in env::args().skip(1) {
        let s = CString::new(arg).unwrap();
        ptrs.push(s.as_ptr() as *mut u8);
        args.push(s);
    }

    unsafe {
        rte_eal_init(ptrs.len() as i32, ptrs.as_ptr() as *mut _);
        let nb_ports = rte_eth_dev_count_avail();
        
        println!("numbers of ports: {}", nb_ports);

        assert!(nb_ports > 0);

        let name = CString::new("default_mbuf_pool").unwrap();
        let num_mbufs = 8191;
        let mbuf_cache_size = 250;
        let mbuf_pool = rte_pktmbuf_pool_create(
            name.as_ptr(),
            (num_mbufs * nb_ports) as u32,
            mbuf_cache_size,
            0,
            RTE_MBUF_DEFAULT_BUF_SIZE as u16,
            rte_socket_id() as i32,
        );
        assert!(!mbuf_pool.is_null());
        let mut port_id = 0;
        let owner = RTE_ETH_DEV_NO_OWNER as u64;
        let mut p = rte_eth_find_next_owned_by(0, owner) as u16;
        while p < RTE_MAX_ETHPORTS as u16 {
            port_id = p;
            initialize_dpdk_port(p, mbuf_pool);
            p = rte_eth_find_next_owned_by(p + 1, owner) as u16;
        }

        let mut m: MaybeUninit<rte_ether_addr> = MaybeUninit::zeroed();
        rte_eth_macaddr_get(port_id, m.as_mut_ptr());
        let link_addr = m.assume_init().addr_bytes;
        println!("Link addr: {:x?}", link_addr);
    };
}

unsafe fn initialize_dpdk_port(port_id: u16, mbuf_pool: *mut rte_mempool) {
    let rx_rings = 1;
    let tx_rings = 1;
    let rx_ring_size = 128;
    let tx_ring_size = 512;
    let nb_rxd = rx_ring_size;
    let nb_txd = tx_ring_size;

    let rx_pthresh = 0;
    let rx_hthresh = 0;
    let rx_wthresh = 0;

    let tx_pthresh = 0;
    let tx_hthresh = 0;
    let tx_wthresh = 0;

    let dev_info = {
        let mut d = MaybeUninit::zeroed();
        rte_eth_dev_info_get(port_id, d.as_mut_ptr());
        d.assume_init()
    };

    let mut port_conf: rte_eth_conf = { MaybeUninit::zeroed().assume_init() };
    port_conf.rxmode.max_rx_pkt_len = RTE_ETHER_MAX_LEN;

    // FIXME: need to check this more details from aws docs. https://github.com/amzn/amzn-drivers/tree/master/userspace/dpdk#11-rss-support 
    // port_conf.rxmode.mq_mode = rte_eth_rx_mq_mode_ETH_MQ_RX_RSS;
    // port_conf.rx_adv_conf.rss_conf.rss_hf = ETH_RSS_IP as u64 | dev_info.flow_type_rss_offloads;
    // port_conf.txmode.mq_mode = rte_eth_tx_mq_mode_ETH_MQ_TX_NONE;

    let mut rx_conf: rte_eth_rxconf = { MaybeUninit::zeroed().assume_init() };
    rx_conf.rx_thresh.pthresh = rx_pthresh;
    rx_conf.rx_thresh.hthresh = rx_hthresh;
    rx_conf.rx_thresh.wthresh = rx_wthresh;
    rx_conf.rx_free_thresh = 32;

    let mut tx_conf: rte_eth_txconf = { MaybeUninit::zeroed().assume_init() };
    tx_conf.tx_thresh.pthresh = tx_pthresh;
    tx_conf.tx_thresh.hthresh = tx_hthresh;
    tx_conf.tx_thresh.wthresh = tx_wthresh;
    tx_conf.tx_free_thresh = 32;

    {
        let ret = rte_eth_dev_configure(port_id, rx_rings, tx_rings, &port_conf as *const _);
        assert_eq!(ret, 0);
    }

    let socket_id = 0;

    {
        for i in 0..rx_rings {
            let ret = rte_eth_rx_queue_setup(port_id, i, nb_rxd, socket_id, &rx_conf as *const _, mbuf_pool);
            assert_eq!(ret, 0);
        }
        for i in 0..tx_rings {
            let ret = rte_eth_tx_queue_setup(port_id, i, nb_txd, socket_id, &tx_conf as *const _);
            assert_eq!(ret, 0);
        }
        assert_eq!(rte_eth_dev_start(port_id), 0);
        rte_eth_promiscuous_enable(port_id);
    }

    if { rte_eth_dev_is_valid_port(port_id) } == 0 {
        panic!("Invalid port");
    }

    let sleep_duration = Duration::from_millis(100);
    let mut retry_count = 90;

    loop {
        {
            let mut link: MaybeUninit<rte_eth_link> = MaybeUninit::zeroed();
            rte_eth_link_get_nowait(port_id, link.as_mut_ptr());
            let link = link.assume_init();
            if link.link_status() as u32 == ETH_LINK_UP {
                let duplex = if link.link_duplex() as u32 == ETH_LINK_FULL_DUPLEX {
                    "full"
                } else {
                    "half"
                };
                eprintln!(
                    "Port {} Link Up - speed {} Mbps - {} duplex",
                    port_id, link.link_speed, duplex
                );
                break;
            }
            rte_delay_us_block(sleep_duration.as_micros() as u32);
        }
        if retry_count == 0 {
            panic!("Link never came up");
        }
        retry_count -= 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::main;

    #[test]
    fn it_works() {
        main();
    }
}