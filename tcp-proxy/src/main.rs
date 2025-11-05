use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};

/// How to use
/// ncat -lk 5001 -c 'xargs -n1 echo Server1:'
/// nc 127.0.0.1 4000

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000")?;
    let backends= Arc::new(vec![
        "127.0.0.1:5001".to_string(),
        "127.0.0.1:5002".to_string(),
    ]);
    let counter = Arc::new(Mutex::new(0));
    for stream in listener.incoming() {
        let stream = stream?;
        let backends = Arc::clone(&backends);
        let counter = Arc::clone(&counter);
        thread::spawn(move || {
            if let Err(e)= handle_connection(stream, &backends, &counter) {
                eprintln!("Error: {}", e);
            }
        });
    }
    Ok(())
}

fn get_next_backend(backends: &Vec<String>, counter: &Mutex<usize>) -> String {
    let mut idx = counter.lock().unwrap();
    let backend = backends[*idx % backends.len()].clone();
    *idx+=1;
    backend
}
fn handle_connection(mut client: TcpStream, backends: &Vec<String>, counter: &Mutex<usize>) -> std::io::Result<()> {
    let backend_address = get_next_backend(backends,counter);
    let mut server = TcpStream::connect(backend_address)?;
    let mut client_clone = client.try_clone()?;
    let mut server_clone = server.try_clone()?;
    let client_to_server= thread::spawn(move || {
        let mut buffer = [0; 512];
        loop{
            let n = client_clone.read(&mut buffer)?;
            if n==0{break;}
            server.write_all(&buffer[..n])?;
        }
        Ok::<(), std::io::Error>(())
    });

    let server_to_client = thread::spawn(move || {
        let mut buffer = [0; 512];
        loop{
            let n = server_clone.read(&mut buffer)?;
            if n==0{break;}
            client.write_all(&buffer[..n])?;
        }
        Ok::<(), std::io::Error>(())
    });
    client_to_server.join().unwrap();
    server_to_client.join().unwrap();
    Ok(())
}