use crate::authentication::schemas::SessionMetadata;
use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::FromRequest;
use std::future::{Ready, ready};

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

    fn update_session(&self, metadata: SessionMetadata) -> Result<(), SessionInsertError> {
        // Very simple replacement
        self.value().purge();
        self.value().insert(Self::ID_KEY, metadata)
    }

    fn insert_sesh_user(&self, metadata: SessionMetadata) -> Result<(), SessionInsertError> {
        self.value().insert(Self::ID_KEY, metadata)
    }

    fn get_sesh_user(&self) -> Result<Option<SessionMetadata>, SessionGetError> {
        self.value().get(Self::ID_KEY)
    }

    fn log_out(&self) {
        self.value().purge();
    }
}

impl SessionType for StaffSession {
    fn value(&self) -> Session {
        self.0.clone()
    }

    fn from_session(session: Session) -> Self {
        StaffSession(session)
    }
}

impl SessionType for CustomerSession {
    fn value(&self) -> Session {
        self.0.clone()
    }

    fn from_session(session: Session) -> Self {
        CustomerSession(session)
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
