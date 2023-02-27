use std::net::SocketAddr;
use socket2::{Socket, Domain, Type};
use std::mem::MaybeUninit;

fn main() {
    let socket: Socket = Socket::new(Domain::IPV6, Type::STREAM, None).unwrap();
    socket.set_only_v6(false).unwrap();
    let send_address: SocketAddr = "[::1]:8001".parse().unwrap();
    let server_address: SocketAddr = "[::1]:8000".parse().unwrap();
    
    socket.bind(&send_address.into()).unwrap();
    socket.connect(&server_address.into()).unwrap();

    match request_file(&socket, "/index.html"){
        Ok(_e) => {},
        Err(e) => {
            println!("{e}");
            return;
        }
    }

    match receive_file(&socket) {
        Ok(e) => println!("{e}"),
        Err(e) => println!("{e}")
    }
}

fn request_file(socket: &Socket, path: &str) -> Result<String, String> { 
    let header = create_request_header(path.to_owned());
    match socket.send(header.as_bytes()) {
        Ok(_a) => return Ok("Packet Sent".to_string()),
        Err(_e) => return Err("Error sending packet".to_string())
    }
}

fn receive_file(socket: &Socket) -> Result<String, String> {  
    let mut buffer = [MaybeUninit::<u8>::uninit(); 512];  
 
    match socket.recv(&mut buffer){
        Ok(a) => {
            if a < 1 {
                return Err("Buffer is 0".to_string());
            };
        }, 
        Err(_e) => {
            return Err("Error Receving files".to_string());
        }
    }
    
    let mut buffer_translated : String = "".to_string();
    for i in buffer {
        unsafe {buffer_translated.push(i.assume_init() as char)}
    }
    return Ok(buffer_translated);

}

fn create_request_header(path: String) -> String { 
        return "GET ".to_owned() + &path + " HTTP/1.1 \r\n
        Host: developer.mozilla.org \r\n
        User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.9; rv:50.0) Gecko/20100101 Firefox/50.0 \r\n
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8 \r\n
        Accept-Language: en-US,en;q=0.5 \r\n
        Accept-Encoding: gzip, deflate, br \r\n
        Referer: https://developer.mozilla.org/testpage.html \r\n
        Connection: keep-alive \r\n
        Upgrade-Insecure-Requests: 1 \r\n
        If-Modified-Since: Mon, 18 Jul 2016 02:36:04 GMT \r\n
        If-None-Match: \"c561c68d0ba92bbeb8b0fff2a9199f722e3a621a\" \r\n
        Cache-Control: max-age=0"; 
}
