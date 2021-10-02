use socket2::SockAddr;
use std::io;
use std::io::Error;
// use std::mem;
use std::net::SocketAddr;
use std::os::unix::io::AsRawFd;
use tokio::net::TcpStream;
// use tokio::net::{TcpListener, TcpSocket};

pub fn get_original_destination_addr(s: &TcpStream) -> io::Result<SocketAddr> {
    let fd = s.as_raw_fd();

    unsafe {
        let (_, target_addr) = SockAddr::init(|target_addr, target_addr_len| {
            match s.local_addr()? {
                SocketAddr::V4(..) => {
                    let ret = libc::getsockopt(
                        fd,
                        libc::SOL_IP,
                        libc::SO_ORIGINAL_DST,
                        target_addr as *mut _,
                        target_addr_len, // libc::socklen_t
                    );
                    if ret != 0 {
                        let err = Error::last_os_error();
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
                        let err = Error::last_os_error();
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

// pub async fn create_redir_listener(addr: SocketAddr) -> io::Result<TcpListener> {
//     let socket = match addr {
//         SocketAddr::V4(..) => TcpSocket::new_v4()?,
//         SocketAddr::V6(..) => TcpSocket::new_v6()?,
//     };

//     // For Linux 2.4+ TPROXY
//     // Sockets have to set IP_TRANSPARENT, IPV6_TRANSPARENT for retrieving original destination by getsockname()
//     unsafe {
//         let fd = socket.as_raw_fd();

//         let enable: libc::c_int = 1;
//         let ret = match addr {
//             SocketAddr::V4(..) => libc::setsockopt(
//                 fd,
//                 libc::IPPROTO_IP,
//                 libc::IP_TRANSPARENT,
//                 &enable as *const _ as *const _,
//                 mem::size_of_val(&enable) as libc::socklen_t,
//             ),
//             SocketAddr::V6(..) => libc::setsockopt(
//                 fd,
//                 libc::IPPROTO_IPV6,
//                 libc::IPV6_TRANSPARENT,
//                 &enable as *const _ as *const _,
//                 mem::size_of_val(&enable) as libc::socklen_t,
//             ),
//         };

//         if ret != 0 {
//             return Err(Error::last_os_error());
//         }
//     }

//     // tokio requires allow reuse addr
//     socket.set_reuseaddr(true)?;

//     // bind, listen as original
//     socket.bind(addr)?;
//     // listen backlogs = 1024 as mio's default
//     socket.listen(1024)
// }
