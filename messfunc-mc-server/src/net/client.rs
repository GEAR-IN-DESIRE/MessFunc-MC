use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;

pub struct Client {
    pub stream: TcpStream,
    pub addr: SocketAddr,
}
impl Client {
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Client {
        stream.set_nonblocking(true).expect("设置stream为不阻塞模式失败");
        Client {
            stream,
            addr,
        }
    }

    pub fn send_packet(&mut self, packet: &[u8]) {
        let r = self.stream.write(packet);
        #[cfg(debug_assertions)]
        r.expect("客户端发送数据包失败");
    }
    
}