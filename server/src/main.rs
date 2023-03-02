use std::fs::File;
use std::io::{Read, stdout, Write};
use std::net::SocketAddr;
use std::sync::{atomic::AtomicBool, Arc};
use std::thread::{self};
use socket2::   {Socket, Domain, Type};
use std::mem::MaybeUninit;
use std::str::{self};
use std::path::Path;

fn main() {
    let mut handles = vec![];

    //sets up sig-int interrupt  
    let quit = Arc::new(AtomicBool::new(false)); 
    let quit_clone = Arc::clone(&quit);

    match ctrlc::set_handler(move || {
        println!("Quitting after finishing request");
        quit_clone.store(true, std::sync::atomic::Ordering::Relaxed); 
        return; 
    }) {
            Ok(_e) => {},
            Err(_e) => {
                println!("Unable to Quit");
                return;
            }
    };
    
    let socket = Socket::new(Domain::IPV6, Type::STREAM, None).unwrap();  
    socket.set_only_v6(false).unwrap();
    let recv_address : SocketAddr = "[::1]:8000".parse().unwrap();
    socket.bind(&recv_address.into()).unwrap();
    
    loop {
        let quit_clone = Arc::clone(&quit);
        socket.listen(128).unwrap();

        let mut clone_socket = socket.try_clone().unwrap();
        let open_request = clone_socket.accept();

        match open_request {
            Ok((sock, _addr)) => {
                clone_socket = sock; 
                let handle = thread::spawn(move || {
                    connection_handler(clone_socket);   
                });
                handles.push(handle);
            }, 
            Err(e) => {
                println!("{e}");
                break;
            }
        }
    
        if quit_clone.load(std::sync::atomic::Ordering::Relaxed) {
            println!("quitting");
            for handle in handles {
                handle.join().unwrap();
            }
            break;
        };
    }
}

fn connection_handler(socket: Socket) {  
    let received_request_result: Result<String, String>;
    let sent_request_result: Result<bool, String>;
    
    received_request_result = receive_request(&socket);
    match received_request_result {
        Ok(path) => sent_request_result = reply_request(&socket, &path),
        Err(e) => {
            println!("43: {e}");
            return;
        }
    }

    match sent_request_result {
        Ok(_e) => println!("Packet Sent"),
        Err(e) => println!("50: {e}")
    }
    stdout().flush().unwrap();
    
}

fn receive_request(socket: &Socket) -> Result<String, String>{
    println!("Received a Request");
    let mut buffer = [MaybeUninit::<u8>::uninit(); 512];  
    let _request: &str;
    let address: &str;

    match socket.recv(&mut buffer) {
        Ok(result) => { 
            if result < 1 {
                return Err("Buffer < 0".to_string());
            }

            let mut line : String = "".to_owned();
            
            for i in buffer {
                unsafe { line.push(i.assume_init() as char) }
            }

            let translated_packet : Vec<&str> = line.split(" ").collect();
            
            _request = translated_packet[0];
            address = translated_packet[1];
            return Ok(address.to_string());

        },
        Err(_e) => {
            return Err("There was an error, accepting packets again".to_string());
        },
    }
}

fn reply_request(socket: &Socket, path: &str) -> Result<bool, String> {
    let mut file: String =  "HTTP/1.1 200 OK \r\nContent-Type: text/html \r\n\r\n".to_string(); 
    match get_file(path) {
        Ok(f) => file += &f.to_string(),
        Err(_e) => return fnf_error_handler(&socket),
    }

    let tcp_response : &[u8] = file.as_bytes();
    
    match socket.send(tcp_response) {
        Ok(_a) => return Ok(true),
        Err(e) => {
            println!("98: {e}");
            return Err("Sending Failed".to_string());},
    }
}

fn get_file(path: &str) -> Result<String, &str> {
    let fixed_path = &path[1..path.len()];

    let to_open = Path::new(fixed_path);
    let mut file = match File::open(&to_open) {
        Ok(file) => file,
        Err(_e) => return Err("Unable to find file")
    };

    let mut file_in_string: String = String::new();
    
    match file.read_to_string(&mut file_in_string) {
        Ok(_a) => return Ok(file_in_string),
        Err(_e) => return Err("Unable to read file")
    };
}

fn fnf_error_handler(socket: &Socket) -> Result<bool, String> {
    let file =  "HTTP/1.1 404 NOT FOUND \r\nContent-Type: text/html \r\n\r\n".to_string(); 
    let tcp_response: &[u8] = file.as_bytes();
    
    match socket.send(tcp_response) {
        Ok(_a) => return Ok(true),
        Err(_e) => return Err("Error sending 404".to_string())
    }
}
