use lettre::{message::{Mailbox, Message, SinglePart}, transport::smtp::PoolConfig, AsyncSmtpTransport, Tokio1Executor, AsyncTransport};
use crate::ports::outputs::mailer::Mailer;
use crate::domain::types::{Error, Value};
use crate::domain::types::Mail;
use crate::ports::Result;


type Client = AsyncSmtpTransport<Tokio1Executor>;

/// This is the default SmtpMailer Client
#[derive(Debug, Clone)]
pub struct SmtpMailer(Client);


impl Mailer for SmtpMailer {
    type Config = Mail;
    /// sender, receiver, subject, body
    type Mail = (Mailbox, Mailbox, String, String);

    async fn new(mail: Self::Config) -> Result<Self> {
        let connection_url = &mail.url;
        let mut mailer = Client::from_url(connection_url).map_err(|err|Error::from(err))?;
        if let Some(credentials) = mail.credentials {
            mailer = mailer.credentials(credentials)
        }
        let client = mailer.pool_config(PoolConfig::new()).build();
        Ok(SmtpMailer(client))
    }

    async fn send(&self, mail: Self::Mail) -> Result<()> {
        let sender = mail.0;
        let receiver = mail.1;
        let subject = mail.2;
        let body = mail.3;
        let part = SinglePart::html(body);
        let email = Message::builder()
        .from(sender)
        .to(receiver)
        .subject(subject)
        .singlepart(part).map_err(|err|Error::from(err))?;
        self.0.send(email).await.map_err(|err|Error::from(err))?;
        Ok(())
    }
}


impl TryFrom<Mail> for SmtpMailer {
    type Error = Box<dyn std::error::Error + 'static>;

    fn try_from(config: Mail) -> std::result::Result<Self, Self::Error> {
        let runtime = tokio::runtime::Runtime::new()?;
        let future = Self::new(config);
        let mailer = runtime.block_on(future)?;
        Ok(mailer)
    }
}