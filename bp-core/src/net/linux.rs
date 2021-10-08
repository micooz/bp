use socket2::SockAddr;
use std::io;
use std::net::SocketAddr;
use std::os::unix::io::RawFd;

pub fn get_original_destination_addr(local_addr: SocketAddr, fd: RawFd) -> io::Result<SocketAddr> {
    unsafe {
        let (_, target_addr) = SockAddr::init(|target_addr, target_addr_len| {
            match local_addr {
                SocketAddr::V4(..) => {
                    let ret = libc::getsockopt(
                        fd,
                        libc::SOL_IP,
                        libc::SO_ORIGINAL_DST,
                        target_addr as *mut _,
                        target_addr_len, // libc::socklen_t
                    );
                    if ret != 0 {
                        let err = io::Error::last_os_error();
                        return Err(err);
                    }
                }
                SocketAddr::V6(..) => {
                    let ret = libc::getsockopt(
                        fd,
                        libc::SOL_IPV6,
                        libc::IP6T_SO_ORIGINAL_DST,
                        target_addr as *mut _,
                        target_addr_len, // libc::socklen_t
                    );

                    if ret != 0 {
                        let err = io::Error::last_os_error();
                        return Err(err);
                    }
                }
            }
            Ok(())
        })?;

        // Convert sockaddr_storage to SocketAddr
        Ok(target_addr.as_socket().expect("SocketAddr"))
    }
}
