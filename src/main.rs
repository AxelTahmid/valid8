use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::Duration;
use trust_dns_resolver::TokioAsyncResolver;

async fn validate_email(email: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Extract domain from the email address
    let domain: Vec<&str> = email.split('@').collect();
    let domain = domain.get(1).ok_or("Invalid email format")?;

    // Step 2: Check DNS MX records
    let resolver = TokioAsyncResolver::tokio_from_system_conf()?;
    let mx_records = resolver.mx_lookup(domain).await?;
    if mx_records.is_empty() {
        return Err("No MX records found for the domain".into());
    }

    // Step 3: Establish an SMTP connection
    let smtp_server = &format!("{}:25", mx_records[0].exchange());
    let smtp_connection = TcpStream::connect(smtp_server).await?;

    // Step 4: Send SMTP commands
    send_smtp_commands(smtp_connection, email).await?;

    Ok(())
}

async fn send_smtp_commands(mut stream: TcpStream, email: &str) -> Result<(), Box<dyn std::error::Error>> {
    let commands = [
        "EHLO example.com\r\n",
        "MAIL FROM: <sender@example.com>\r\n",
        &format!("RCPT TO: <{}>\r\n", email),
        "DATA\r\n",
        "Subject: Test email\r\n",
        "This is a test email.\r\n",
        ".\r\n",
    ];

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

#[tokio::main]
async fn main() {
    let email_address = "test@example.com";

    match validate_email(email_address).await {
        Ok(_) => println!("Email address is valid"),
        Err(err) => eprintln!("Validation failed: {}", err),
    }
}
