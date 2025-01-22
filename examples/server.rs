use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut db = rustore::Database::open("example.db").unwrap();

    println!("Server listening on port 8080");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];

                'read_loop: while match stream.read(&mut buffer) {
                    Ok(n) if n == 0 => false,
                    Ok(n) => {
                        let message = String::from_utf8_lossy(&buffer[..n]);
                        let tokens = message.split_whitespace().collect::<Vec<&str>>();

                        if tokens.is_empty() {
                            continue 'read_loop;
                        }

                        match tokens[0].to_lowercase().as_str() {
                            "get" => {
                                let key = tokens[1];
                                let result = db.get(key);
                                match result {
                                    Ok(value) => {
                                        stream.write(value.as_bytes()).unwrap();
                                    }
                                    Err(_) => {
                                        stream.write("Key not found \n".as_bytes()).unwrap();
                                    }
                                }
                            }
                            "put" => {
                                let key = tokens[1];
                                let value = tokens[2];
                                db.put(key, value).unwrap();
                                stream.write("OK".as_bytes()).unwrap();
                            }
                            "delete" => {
                                let key = tokens[1];
                                db.delete(key).unwrap();
                                stream.write("OK".as_bytes()).unwrap();
                            }
                            _ => {
                                stream.write("Invalid command".as_bytes()).unwrap();
                            }
                        }

                        true
                    }
                    Err(_) => false,
                } {}
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
