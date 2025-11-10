use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::FromRequest;
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use std::future::{Ready, ready};
use uuid::Uuid;

use crate::authentication::token::{SessionClaims, TokenHandler, TokenPair};
use crate::base::Email;
use crate::infra::redis::RedisPool;
use crate::user::models::AccessRole;

pub struct StaffSession(Session);

pub struct CustomerSession(Session);

pub trait SessionType: FromRequest
where
    Self::Error: Into<actix_web::Error>,
{
    const ID_KEY: &'static str = "user";

    fn value(&self) -> Session;
    fn from_session(session: Session) -> Self;

    fn renew(&self) {
        self.value().renew();
    }

    fn update_session(&self, metadata: &SessionClaims) -> Result<(), SessionInsertError> {
        // Very simple replacement
        self.value().purge();
        self.value().insert(Self::ID_KEY, metadata)
    }

    fn insert_sesh_user(&self, metadata: &SessionClaims) -> Result<(), SessionInsertError> {
        self.value().insert(Self::ID_KEY, metadata)
    }

    fn get_sesh_user(&self) -> Result<Option<SessionClaims>, SessionGetError> {
        self.value().get(Self::ID_KEY)
    }

    fn log_out(&self) {
        self.value().purge();
    }

    fn kind(&self) -> &str;
}

impl SessionType for StaffSession {
    fn value(&self) -> Session {
        self.0.clone()
    }

    fn from_session(session: Session) -> Self {
        StaffSession(session)
    }

    fn kind(&self) -> &str {
        "Staff"
    }
}

impl SessionType for CustomerSession {
    fn value(&self) -> Session {
        self.0.clone()
    }

    fn from_session(session: Session) -> Self {
        CustomerSession(session)
    }

    fn kind(&self) -> &str {
        "Customer"
    }
}

impl FromRequest for CustomerSession {
    type Error = <Session as FromRequest>::Error;
    type Future = Ready<Result<CustomerSession, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(Ok(CustomerSession::from_session(req.get_session())))
    }
}

impl FromRequest for StaffSession {
    type Error = <Session as FromRequest>::Error;
    type Future = Ready<Result<StaffSession, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(Ok(StaffSession::from_session(req.get_session())))
    }
}

pub async fn session_handler<T: SessionType>(
    token_handler: &TokenHandler,
    redis_pool: &RedisPool,
    session: &T,
    id: Uuid,
    email: Email,
    role: AccessRole,
) -> Result<(TokenPair, SessionClaims), anyhow::Error> {
    let (pair, session_claims) = token_handler
        .generate_tokens(redis_pool, id, email, role)
        .await?;

    session.renew();

    session
        .insert_sesh_user(&session_claims)
        .context(format!("Failed to insert new session {}", session.kind()))?;

    FlashMessage::success(format!("{} authorized", session.kind())).send();

    Ok((pair, session_claims))
}
