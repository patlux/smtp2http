use lettre::message::header::ContentType;
use lettre::{Message, SmtpTransport, Transport as _};
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
    SendTestMail {
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
        let mail = mail_parser::MessageParser::default()
            .parse(&message)
            .unwrap();

        let body = serde_json::to_string(&mail).unwrap();

        let client = reqwest::blocking::Client::new();
        let result = client.post(&self.endpoint).body(body).send();

        match result {
            Ok(_) => println!("Forwarded mail to {}.", &self.endpoint),
            Err(err) => println!("Forwaring failed: {}", err),
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
        Commands::SendTestMail { url } => {
            let email = Message::builder()
                .from("NoBody <nobody@domain.tld>".parse().unwrap())
                .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
                .to("Hei <hei@domain.tld>".parse().unwrap())
                .subject("Happy new year")
                .header(ContentType::TEXT_PLAIN)
                .body(String::from("Be happy!"))
                .unwrap();

            let result = SmtpTransport::from_url(&url).unwrap().build().send(&email);

            match result {
                Ok(_) => info!("Email sent successfully!"),
                Err(e) => panic!("Could not send email: {e:?}"),
            }
        }
        Commands::Server { listen, endpoint } => {
            info!(r#"Forwards mails to "{}"."#, &endpoint);
            start_smtp2http(listen, endpoint).expect("Failed to start smtp server");
            info!("SMTP server closed.");
        }
    }

    Ok(())
}

fn start_smtp2http(listen: String, endpoint: String) -> Result<(), mailin_embedded::err::Error> {
    let mut server = Server::new(Smtp2HttpHandler {
        data: vec![],
        endpoint: endpoint.clone(),
    });

    server
        .with_name("localhost")
        .with_ssl(SslConfig::None)
        .unwrap()
        .with_addr(listen)?;

    server.serve()
}
