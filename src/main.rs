use lettre::message::header::ContentType;
use lettre::{Message, SmtpTransport, Transport};
use mailin_embedded::{response::OK, Handler, Server, SslConfig};
use structopt::StructOpt;
use tracing::info;

#[derive(StructOpt)]
#[structopt()]
enum Commands {
    Server {
        #[structopt(default_value = "127.0.0.1:25")]
        listen: String,
    },
    Send {
        #[structopt(default_value = "smtp://127.0.0.1:25")]
        url: String,
    },
}

#[derive(Clone)]
struct HttpForwarderHandler {
    pub data: Vec<u8>,
}

impl Handler for HttpForwarderHandler {
    fn data(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.data.append(&mut buf.to_owned());
        Ok(())
    }
    fn data_end(&mut self) -> mailin::Response {
        let message = String::from_utf8(self.data.to_vec()).unwrap();
        info!("{}", message);
        OK
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let opt = Commands::from_args();

    match opt {
        Commands::Send { url } => {
            let email = Message::builder()
                .from("NoBody <nobody@domain.tld>".parse().unwrap())
                .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
                .to("Hei <hei@domain.tld>".parse().unwrap())
                .subject("Happy new year")
                .header(ContentType::TEXT_PLAIN)
                .body(String::from("Be happy!"))
                .unwrap();

            let mailer = SmtpTransport::from_url(&url).unwrap().build();

            match mailer.send(&email) {
                Ok(_) => info!("Email sent successfully!"),
                Err(e) => panic!("Could not send email: {e:?}"),
            }
        }
        Commands::Server { listen } => {
            let smtp2http_handler = HttpForwarderHandler { data: vec![] };
            let mut server = Server::new(smtp2http_handler);

            server
                .with_name("localhost")
                .with_ssl(SslConfig::None)
                .unwrap()
                .with_addr(listen)
                .unwrap();

            info!("Start server ...");
            server.serve().unwrap();

            info!("Server closed.");
        }
    }

    Ok(())
}
