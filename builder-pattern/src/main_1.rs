mod main;

struct Server{
    host:String,
    port:u16,
    ssl:bool
}

struct ServerBuilder{
    host: Option<String>,
    port: Option<u16>,
    ssl: Option<bool>
}

impl ServerBuilder{
    fn new()->Self{
        ServerBuilder{
            host:None,
            port:None,
            ssl: None
        }
    }

    fn host(mut self, host:&str) ->Self{
        self.host=Some(host.to_string());
        self
    }

    fn port(mut self,port:u16)->Self{
        self.port=Some(port);
        self
    }

    fn ssl(mut self,ssl:bool)->Self{
        self.ssl=Some(ssl);
        self
    }

    fn build(self)->Result<Server,&'static str>{
        let host = self.host.ok_or("host is missing")?;
        let port=self.port.ok_or("port is missing")?;
        let ssl = self.ssl.unwrap_or(false);
        Ok(Server{
            host,
            port,
            ssl
        })
    }
}


fn main() {
    let server = ServerBuilder::new()
                .host("localhost")
                .port(8080)
                .ssl(true)
                .build()
                .expect("failed to build server");
    println!("server running on {} port {} with ssl set to {}", server.host,server.port,server.ssl);
}
