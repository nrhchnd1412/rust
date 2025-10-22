enum ConnectionState{
    Disconnected,
    Connected {session_id: String},
    Failed {error: String},
}

struct Connection {
    state: ConnectionState,
}

impl Connection {
    fn new(session_id: String) -> Connection {
        Connection{
            state: ConnectionState::Disconnected,
        }
    }

    fn connect(&mut self, session_id: String) {
        self.state = ConnectionState::Connected {session_id };
    }

    fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    fn status(&mut self) -> String {
        match &self.state {
            ConnectionState::Disconnected => { "disconnected".to_string() },
            ConnectionState::Connected {session_id}=> { "connected".to_string() },
            ConnectionState::Failed {error} => { "failed".to_string() },
        }
    }
}


fn main() {
    let mut connection = Connection::new("session".to_string());
    println!("{}", connection.status());
    connection.connect("session".to_string());
    println!("{}", connection.status());
    connection.disconnect();
    println!("{}", connection.status());
}
