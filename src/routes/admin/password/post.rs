use crate::utils::{e500, see_other};
use crate::authentication::{validate_credentials, AuthError, Credentials, UserId};
use crate::routes::admin::dashboard::get_username;
use actix_web::{HttpResponse, web};
use secrecy::Secret;
use sqlx::PgPool;
use secrecy::ExposeSecret;
use actix_web_flash_messages::FlashMessage;

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>, 
}


pub async fn change_password(
    form: web::Data<FormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        return Ok(see_other("/admin/password"));
    }
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
    }
    let new_password = form.new_password.expose_secret();
    if !is_password_strong(&new_password) {
        FlashMessage::error("Password must be between 12 and 128 characters long.").send();
        return Ok(see_other("/admin/password"));
    }
    let username = get_username(*user_id, &pool).await.map_err(e500)?;

    let credentials = Credentials {
        username,
        password: form.current_password.clone(),
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(e500(e).into()),
        }
    }
    crate::authentication::change_password(*user_id, form.new_password.clone(), &pool)
        .await
        .map_err(e500)?;
    FlashMessage::error("Your password has been changed.").send();
    Ok(see_other("/admin/password"))
}

fn is_password_strong(password: &str) -> bool {
    password.len() >= 12 && password.len() <= 128
}

