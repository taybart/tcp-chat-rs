use clap::{App, Arg};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let matches = App::new("chat_example")
        .version("0.1")
        .author("Taylor B. <taybart@gmail.com>")
        .about("A tcp chat server")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("ADDRESS")
                .help("The address of the eyeball that we will be proxying traffic for")
                .takes_value(true)
                .default_value("8080"),
        )
        .get_matches();

    let port = matches.value_of("port").unwrap();

    // bind tcp listener
    let listener = TcpListener::bind(format!("localhost:{}", port))
        .await
        .unwrap();

    let (tx, _rx) = broadcast::channel(10);

    loop {
        // accept tcp connection
        let (mut socket, addr) = listener.accept().await.unwrap();

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // move this to new task
        tokio::spawn(async move {
            // split socket into reader/writer so that we can move it in the loop
            let (reader, mut writer) = socket.split();

            // tokio::BufReader different than std::BufReader
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                // special tokio select handler similar to go
                tokio::select! {
                    // recieve from the socket of the client
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        // broadcast to attached clients
                        // tx.send(line.clone()).unwrap();

                        // broadcast to attached clients with addr
                        tx.send((line.clone(), addr)).unwrap();
                        // Must clear line when we are done with the message
                        line.clear()
                    }
                    // broadcast recv
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        // don't send back to original, small "hack" for telnet clients
                        if addr != other_addr {
                            // echo message back to client socket
                            writer.write_all(&msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
