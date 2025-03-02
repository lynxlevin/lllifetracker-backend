use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        PoolConfig,
    },
    Message, SmtpTransport, Transport,
};

// MYMEMO: refactor
#[tracing::instrument(
    name = "Generic e-mail sending function.",
    skip(recipient_email, recipient_first_name, recipient_last_name, subject, html_content, text_content),
    fields(recipient_email = %recipient_email, recipient_first_name = %recipient_first_name, recipient_last_name = %recipient_last_name)
)]
pub async fn send_email(
    recipient_email: String,
    recipient_first_name: String,
    recipient_last_name: String,
    subject: impl Into<String>,
    html_content: impl Into<String>,
    text_content: impl Into<String>,
) -> Result<(), String> {
    let settings = settings::get_settings();

    let email = Message::builder()
        .from(match settings.email.sender.parse() {
                Ok(mailbox) => mailbox,
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "Failed to get sender mailbox setting: {:#?}", e);
                    return Err(e.to_string());
                }
            },
        )
        .to(format!(
            "{} <{}>",
            [recipient_first_name, recipient_last_name].join(" "),
            recipient_email
        )
        .parse()
        .unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text_content.into()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html_content.into()),
                ),
        )
        .unwrap();

    let credentials = Credentials::new(settings.email.host_user, settings.email.host_user_password);
    let sender = SmtpTransport::starttls_relay(&settings.email.host)
        .unwrap()
        .credentials(credentials)
        .authentication(vec![Mechanism::Plain])
        .pool_config(PoolConfig::new().max_size(20))
        .build();

    match sender.send(&email) {
        Ok(_) => {
            tracing::event!(target: "backend", tracing::Level::INFO, "Email successfully sent!");
            Ok(())
        }
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "Could not send email: {:#?}", e);
            Err(format!("Could not send email: {:#?}", e))
        }
    }
}

// MYMEMO: refactor
#[tracing::instrument(
    name = "Generic multipart e-mail sending function.",
    skip(redis_connection),
    fields(
        recipient_user_id = %user_id,
        recipient_email = %recipient_email,
        recipient_first_name = %recipient_first_name,
        recipient_last_name = %recipient_last_name,
    )
)]
pub async fn send_multipart_email(
    subject: String,
    user_id: uuid::Uuid,
    recipient_email: String,
    recipient_first_name: String,
    recipient_last_name: String,
    template_name: &str,
    redis_connection: &mut deadpool_redis::Connection,
) -> Result<(), String> {
    let settings = settings::get_settings();
    let title = format!("Lynx Levin's LifeTracker - {subject}");

    let issued_token = match crate::auth::tokens::issue_confirmation_token_pasetors(
        user_id,
        redis_connection,
        None,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
            return Err(format!("{}", e));
        }
    };

    let web_address = {
        if settings.debug {
            format!(
                "{}:{}",
                settings.application.base_url, settings.application.port
            )
        } else {
            settings.application.base_url
        }
    };

    let confirmation_link = {
        if template_name == "password_reset_email.html" {
            format!(
                "{}/users/password-change/email-verification?token={}",
                web_address, issued_token
            )
        } else {
            format!(
                "{}/users/register/confirm?token={}",
                web_address, issued_token,
            )
        }
    };

    let current_date_time = chrono::Local::now();
    let dt = current_date_time + chrono::Duration::minutes(settings.secret.token_expiration);

    let template = settings::ENV.get_template(template_name).unwrap();
    let ctx = minijinja::context! {
        title => &title,
        confirmation_link => &confirmation_link,
        domain => &settings.frontend_url,
        expiration_time => &settings.secret.token_expiration,
        exact_time => &dt.format("%A %B %d, %Y at %r").to_string()
    };
    let html_text = template.render(ctx).unwrap();

    let text = format!(
        r#"
        Tap the link below to confirm your email address.
        {}
        "#,
        confirmation_link
    );

    actix_web::rt::spawn(send_email(
        recipient_email,
        recipient_first_name,
        recipient_last_name,
        subject,
        html_text,
        text,
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn send_email() -> Result<(), String> {
        todo!();
    }
    #[actix_web::test]
    #[ignore]
    async fn send_multipart_email() -> Result<(), String> {
        todo!();
    }
}
