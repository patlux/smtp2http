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

        #[structopt(default_value = "http://localhost:1337")]
        endpoint: String,
    },
    Send {
        #[structopt(default_value = "smtp://127.0.0.1:25")]
        url: String,
    },
}

#[derive(Clone)]
struct Smtp2HttpHandler {
    pub data: Vec<u8>,
    pub endpoint: String,
}

impl Handler for Smtp2HttpHandler {
    fn data(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.data.extend_from_slice(buf);
        Ok(())
    }
    fn data_end(&mut self) -> mailin::Response {
        let message = String::from_utf8(self.data.to_vec()).unwrap();

        let client = reqwest::blocking::Client::new();
        let result = client
            .post(&self.endpoint)
            .body(serde_json::json!({ "hello": "sir" }).to_string())
            .send();

        match result {
            Ok(_) => println!("Sent message to {}.", &self.endpoint),
            Err(err) => println!("Sending not successful. {}", err),
        }

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
        Commands::Server { listen, endpoint } => {
            let mut server = Server::new(Smtp2HttpHandler {
                data: vec![],
                endpoint,
            });

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
