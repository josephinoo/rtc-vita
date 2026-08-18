use crate::ifaces::{Interface, Kind};
use std::ffi::CStr;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::raw::{c_char, c_int, c_uint};
use std::str::FromStr;

// PS Vita SceNetCtl constants (from psp2/net/netctl.h in vitasdk)
const SCE_NETCTL_INFO_GET_IP_ADDRESS: c_int = 14;
const SCE_NETCTL_INFO_GET_NETMASK: c_int = 15;

#[repr(C)]
pub union SceNetCtlInfo {
    pub size: c_uint,
    pub ip_address: [c_char; 16],
    pub netmask: [c_char; 16],
    _pad: [u32; 33], // 132 bytes to safely match SceNetCtlInfo C union size
}

#[link(name = "SceNetCtl_stub")]
unsafe extern "C" {
    fn sceNetCtlInetGetInfo(code: c_int, info: *mut SceNetCtlInfo) -> c_int;
}

/// Enumerates local network interfaces on PlayStation Vita.
///
/// Queries `SceNetCtl` for the active Wi-Fi connection IP address and subnet mask.
/// If network is uninitialized or disconnected, safely returns an empty vector without panicking.
pub fn ifaces() -> Result<Vec<Interface>, io::Error> {
    let mut interfaces = Vec::new();
    let mut info_ip = unsafe { std::mem::zeroed::<SceNetCtlInfo>() };

    // Query active IP address
    let ret = unsafe { sceNetCtlInetGetInfo(SCE_NETCTL_INFO_GET_IP_ADDRESS, &mut info_ip) };
    if ret < 0 {
        // Network module uninitialized or Wi-Fi disconnected: return empty list gracefully
        return Ok(interfaces);
    }

    // Query netmask
    let mut info_mask = unsafe { std::mem::zeroed::<SceNetCtlInfo>() };
    let ret_mask = unsafe { sceNetCtlInetGetInfo(SCE_NETCTL_INFO_GET_NETMASK, &mut info_mask) };

    let ip_str = unsafe { CStr::from_ptr(info_ip.ip_address.as_ptr()) }
        .to_str()
        .unwrap_or("");

    if let Ok(ipv4) = Ipv4Addr::from_str(ip_str) {
        let addr = SocketAddr::new(IpAddr::V4(ipv4), 0);

        let mask = if ret_mask >= 0 {
            let mask_str = unsafe { CStr::from_ptr(info_mask.netmask.as_ptr()) }
                .to_str()
                .unwrap_or("");
            Ipv4Addr::from_str(mask_str)
                .ok()
                .map(|m| SocketAddr::new(IpAddr::V4(m), 0))
        } else {
            None
        };

        interfaces.push(Interface {
            name: "wlan0".to_string(),
            kind: Kind::Ipv4,
            addr: Some(addr),
            mask,
            hop: None,
        });
    }

    Ok(interfaces)
}
