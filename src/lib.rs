#![feature(duration_float)]
use std::collections::{VecDeque, HashMap};
use std::io;
use std::io::prelude::*;
use std::sync::{Mutex, Arc, Condvar};
use std::thread;
use std::net::Ipv4Addr;

mod tcp;


const SENDQUEUE_SIZE: usize = 1024;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

#[derive(Default)]
struct Foobar {
    conn_manager: Mutex<ConnectionManager>,
    pending_var: Condvar,
    rcv_var: Condvar,
}

type InterfaceHandle = Arc<Foobar>;

pub struct Interface{
    interface_handle: Option<InterfaceHandle>,    // lock todo read & writes
    thread_handle: Option<thread::JoinHandle<io::Result<()>>>,
}

impl Drop for Interface {
    fn drop(&mut self) {
        self.interface_handle.as_mut().unwrap().conn_manager.lock().unwrap().terminate = true;
        
        drop(self.interface_handle.take());
        self.thread_handle
            .take()
            .expect("interface dropped more than once")
            .join()
            .unwrap()
            .unwrap();
    }
}


#[derive(Default)]
struct ConnectionManager {
    terminate: bool,
    connections: HashMap<Quad, tcp::Connection>,

    //
    //  这个hash中的的port，就是我们监听的端口，
    //      而 VecDeque 则是现有的连接，这里包含还没有完成三次握手的连接
    //
    pendding: HashMap<u16/*listen port*/, VecDeque<Quad>>,
}

fn packet_loop(mut nic: tun_tap::Iface, ih: InterfaceHandle) -> io::Result<()> {
    let mut buf = [0u8; 1504];
    
    loop{
        // we want to read from nic, 
        // but make sure that will wake up when the next timer has to be riggered !
        use std::os::unix::io::AsRawFd;
        let mut pfd = [nix::poll::PollFd::new(
            nic.as_raw_fd(),
            nix::poll::EventFlags::POLLIN
        )];

        let n = nix::poll::poll(&mut pfd[..], 1000).map_err(|e| e.as_errno().unwrap())?;
        assert_ne!(n, -1);
        
        if n == 0 {
            let mut cmg = ih.conn_manager.lock().unwrap();
            for connection in cmg.connections.values_mut() {
                // TODO: don't die on errors ?
                connection.on_tick(&mut nic)?;
            }

            continue;
        }
        assert_eq!(n, 1);
	    let nbytes = nic.recv(&mut buf[..])?;
		
		// if s/without_packet_info/new/:

		// let _eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
		// let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
		// if eth_proto != 0x0800 {
		//	// no ipv4, link level protocol
		//	continue;
		// }
		// and also include on send

		// 解析ip header
		match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]){
			Ok(iph) => {
				let src = iph.source_addr();
				let dst = iph.destination_addr();
				if iph.protocol() != 0x06 {
					println!("NONE TCP PRPTOCOL");
                    continue;
				}
			
			    // 解析tcp header
			    match etherparse::TcpHeaderSlice::from_slice(&buf[iph.slice().len()..nbytes]) {
				
                    Ok(tcp_header) => {
                        use std::collections::hash_map::Entry;
                        let datai = iph.slice().len() + tcp_header.slice().len();
                        let mut cmg = ih.conn_manager.lock().unwrap();
                        let cm = &mut *cmg;
                        let quad = Quad {
                            src: (src, tcp_header.source_port()),
                            dst: (dst, tcp_header.destination_port()),
                        };

                        match cm.connections.entry(quad) {
                                
                            Entry::Occupied(mut c) => {
                                let a = c.get_mut().on_packet(
                                    &mut nic,
                                    iph,
                                    tcp_header,
                                    &buf[datai..nbytes]
                                )?;
                                drop(cmg);
                                if a.contains(tcp::Available::READ) {
                                    ih.rcv_var.notify_all();
                                }
                                if a.contains(tcp::Available::WRITE) {
                                    //ih.snd _var.notify_all();
                                }
                            },

                            Entry::Vacant(e) => {
                                if let Some(pendding) = cm.pendding.get_mut(&tcp_header.destination_port()) {
                                    if let Some(c) = tcp::Connection::accept(
                                        &mut nic,
                                        iph,
                                        tcp_header,
                                        &buf[datai..nbytes]) ? 
                                    {
                                        e.insert(c);
                                        pendding.push_back(quad);
                                        drop(cmg);
                                        ih.pending_var.notify_all();
                                    }
                                }
                            }
                        }       
                        // (srcip, srcport, dstip, dstport)
                    },

                    Err(e) => {
                        // eprintln!("ignore none tcp header {:?}", e)
                    }
				}
			},
			Err(e) => {
				eprintln!("ignore weird packet {:?}", e)
			}
	    }
	}
}

impl Interface {
    pub fn new() -> io::Result<Self> {
        let nic = tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun)?;
        
        //
        //  Arc: Atomic Reference Counter
        //      原子计数是一种能够让你以线程安全的方式修改和增加它的值的类型
        //      需要注意的是，Arc只能包含不可变数据。
        //          这是因为如果两个线程试图在同一时间修改被包含的值，Arc无法保证避免数据竞争。
        //          如果你希望修改数据，你应该在Arc类型内部封装一个互斥锁保护（Mutex guard）。
        //
        let ih: InterfaceHandle = Arc::default();

        let jh = {
            // 注意我们这里显示的调用 clone, 是因为 主线程 和 packet_loop线程 都需要
            let ih = ih.clone();
            thread::spawn(move || {
                packet_loop(nic, ih)
            })
        };

        Ok(Interface{
            interface_handle: Some(ih),
            thread_handle: Some(jh),
        })
    }

    pub fn bind(&mut self, port: u16) -> io::Result<TcpListener> {
        
        use std::collections::hash_map::Entry;
        
        let mut cm = self.interface_handle.as_mut().unwrap().conn_manager.lock().unwrap();
        
        match cm.pendding.entry(port) {

            Entry::Vacant(v) => {
                // 绑定一个端口，就是初始化一个环形队列，用来存放已经握手完成的连接
                v.insert(VecDeque::new());
            }

            Entry::Occupied(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "port already bound"
                ));
            }
        };

        drop(cm);
        
        Ok(TcpListener {
            port, 
            interface_handle: self.interface_handle.as_mut().unwrap().clone(),
        })
    }
}

//
//  一个 listener 的核心，其实就是那个监听的端口
//    然后，所有连接到这个端口的连接，放到队列里边
//
pub struct TcpListener {
    port: u16,
    interface_handle: InterfaceHandle,
}

impl Drop for TcpListener {

    fn drop(&mut self) {
        let mut cm = self.interface_handle.conn_manager.lock().unwrap();
        
        let pending = cm.pendding
            .remove(&self.port)
            .expect("port closed while listenner still active");

        for _quad in pending {
            // tODO teminate cm.connecions[quad]
            unimplemented!();
        }
    }
}

impl TcpListener {

    pub fn accept(&mut self) -> io::Result<TcpStream> {

        //
        //  简单理解 if let
        //  if let 是 rust 的语法糖， 意思是 if match then let ，match 被简化到只匹配一种情况、同时匹配后绑定
        //  比如：if let Some(x) = optV  { dosth(x);}, 意思是
        //    if (match Some(x)== optV) then {x = v;  dosth(x);｝
        //
        let mut connection_manager = self.interface_handle.conn_manager.lock().unwrap();
        
        loop {
            if let Some(quad) = connection_manager
                .pendding
                .get_mut(&self.port)
                .expect("port closed while listenner still active")
                .pop_front() 
            {
                dbg!("accep fetch ready connections", quad);
                return Ok(TcpStream {
                    quad,
                    h: self.interface_handle.clone(),
                });
            }
            else 
            {
              //panic!("fadsf");
              dbg!("~~~~ if let, connection_manager keys {}", connection_manager.pendding.keys());
            }
            
            connection_manager = self.interface_handle.pending_var.wait(connection_manager).unwrap();
        }
    }
}

pub struct TcpStream {
    quad: Quad,
    h: InterfaceHandle,
}


impl Drop for TcpStream {
    fn drop(&mut self) {
        let cm = self.h.conn_manager.lock().unwrap();
        // TODO:send fin on cmd.connections[quad]
        // if let Some(c) = cm.connections.remove(&self.quad) {    
        //     //unimplemented!();
        // }
    }
}

impl Read for TcpStream{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut cm = self.h.conn_manager.lock().unwrap();
        loop {
            let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::ConnectionAborted, 
                    "stream was terminated unexpectedly!"
                )
            })?;

            if c.is_rcv_closed() && c.incoming.is_empty() {    
                // peer closed
                return Ok(0);
            }
            
            if !c.incoming.is_empty() {
                // TODO: detect fin and return nread == 0
                let mut nread = 0;
                let (head, tail) = c.incoming.as_slices();
                let hread = std::cmp::min(buf.len(), head.len());
                buf[..hread].copy_from_slice(&head[..hread]);
                nread += hread;
                let tread = std::cmp::min(buf.len()-nread, tail.len());
                buf[hread..(hread+tread)].copy_from_slice(&tail[..tread]);
                nread += tread;
                drop(c.incoming.drain(..nread));
                return Ok(nread);
            }
            
            cm = self.h.rcv_var.wait(cm).unwrap();
        }
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut cm = self.h.conn_manager.lock().unwrap();
        let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted, 
                "stream was terminated unexpectedly!"
            )
        })?;

        if c.unacked.len() > SENDQUEUE_SIZE{
            // TODO: block
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "too many bytes buffered",
            ));
        }

        let nwrite = std::cmp::min(buf.len(), SENDQUEUE_SIZE - c.unacked.len());
        c.unacked.extend(buf[..nwrite].iter());
        Ok(nwrite)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        let mut cm = self.h.conn_manager.lock().unwrap();
        let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted, 
                "stream was terminated unexpectedly!"
            )
        })?;

        if c.unacked.is_empty() {
            Ok(())
        } else {
            // TODO: block
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "too many bytes buffered",
            ))
        }
    }
}

impl TcpStream {
    pub fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
        let mut cm = self.h.conn_manager.lock().unwrap();
        let c = cm.connections.get_mut(&self.quad).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::ConnectionAborted, 
                "stream was terminated unexpectedly!"
            )
        })?;
        
        c.close()
    }
}