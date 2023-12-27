use email_address::EmailAddress;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::Duration;
use trust_dns_resolver::TokioAsyncResolver;

#[tokio::main]
async fn main() {
    let email_address = "test@example.com";

    match validate_email(email_address).await {
        Ok(_) => println!("Email address is valid"),
        Err(err) => eprintln!("Validation failed: {}", err),
    }
}

async fn validate_email(email: &str) -> Result<(), EmailVerificationError> {
    // Step 1: Syntax validation using email_address
    let parsed_email = EmailAddress::from_str(email)?;

    // Step 2: MX record lookup (unchanged)
    let resolver = TokioAsyncResolver::tokio_from_system_conf()?;
    let mx_records = resolver.mx_lookup(parsed_email.domain()).await?;
    if mx_records.is_empty() {
        return Err(EmailVerificationError::NoMxRecords);
    }

  // Step 3: SMTP connection (unchanged)
    let smtp_server = &format!("{}:25", mx_records[0].exchange());
    let smtp_connection: TcpStream = TcpStream::connect(smtp_server).await?;


    // Step 4: Send SMTP commands
    let commands = [
        &format!("EHLO {}", parsed_email.local_part()), // Use recipient's local part for EHLO
        "MAIL FROM: <sender@example.com>\r\n",
        &format!("RCPT TO: <{}>\r\n", email),
    ];
    send_smtp_commands(smtp_connection, commands).await?;

    Ok(())
}

async fn send_smtp_commands(mut stream: TcpStream, commands: &str) -> Result<(), Box<dyn std::error::Error>> {
    
     for command in &commands {
        tokio::time::sleep(Duration::from_millis(100)).await; // Delay to avoid rate limiting
        stream.write_all(command.as_bytes()).await?;

        let mut buffer = [0u8; 1024];
        let _ = stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer);
        println!("{}", response);

        // Check for the expected response code (e.g., 250 for success)
        if !response.starts_with("250") {
            return Err(format!("SMTP command failed: {}", response).into());
        }
    }

    Ok(())
}


#[derive(Debug)]
enum EmailVerificationError {
    InvalidEmailFormat,
    NoMxRecords,
    SmtpConnectionError(Box<dyn std::error::Error>),
    SmtpCommandError(String),
}

impl std::fmt::Display for EmailVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidEmailFormat => f.write_str("Invalid email format"),
            Self::NoMxRecords => f.write_str("No MX records found for the domain"),
            Self::SmtpConnectionError(err) => f.write_fmt(format_args!("SMTP connection error: {}", err)),
            Self::SmtpCommandError(err) => f.write_fmt(format_args!("SMTP command error: {}", err)),
        }
    }
}

impl std::error::Error for EmailVerificationError {}