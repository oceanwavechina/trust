use std::{io, thread};
use std::io::prelude::*;


//
//  玩法是这样的：
//      interface 层面是处理底层连接的，有一个 thread 在那 loop, 
//      我们的 tcp_listener 里边有一个队列，保存着三次握手完成的 connection
//



fn main() -> io::Result<()> {

	let mut interface = trust::Interface::new()?;
  
	let mut tcp_listener = interface.bind(9000)?;

  //
  //  spawn 里边是一个闭包， 没有参数
  //    关键字move的作用是将所引用的变量的所有权转移至闭包内，
  //    通常用于使闭包的生命周期大于所捕获的变量的原生命周期（例如将闭包返回或移至其他线程）
  //
	let handler = thread::spawn(move || {

		while let Ok(mut stream) = tcp_listener.accept() {

			dbg!("got connection on 9000 !");
			stream.write(b"hello from server");
			stream.shutdown(std::net::Shutdown::Write).unwrap();
			loop {
				let mut buf = [0; 512];
				let n = stream.read(&mut buf[..]).unwrap();
				if n == 0 {
					eprintln!("no more data");
					break;
				} else {
					println!("{}", std::str::from_utf8(&buf[..n]).unwrap());
				}
			}
			
		}
	});
	handler.join().unwrap();
	Ok(())
}
