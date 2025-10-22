#[derive(Debug)]
struct Server{
    host: String,
    port:u16,
    max_connections:usize,
    timeout:u64
}

struct ServerBuilder{
    host: String,
    port:u16,
    max_connections:usize,
    timeout:u64
}

impl ServerBuilder{
    fn host(mut self,host:&str)->Self{
        self.host=host.to_string();
        self
    }
    fn port(mut self,port:u16)->Self{
        self.port=port;
        self
    }

    fn build(self)->Server{
        Server{
            host: self.host,
            port: self.port,
            max_connections: self.max_connections,
            timeout: self.timeout
        }
    }
}

impl Default for ServerBuilder {
    fn default()->Self{
        ServerBuilder{
            host: String::from("localhost"),
            port: 8080,
            max_connections: 100,
            timeout: 10
        }
    }
}

impl Server{
    fn builder()->ServerBuilder{
        ServerBuilder::default()
    }
}

fn main(){
    let server = Server::builder()
        .port(8080)
        .host("localhost")
        .build();
    println!("{:?}",server);
}