use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::FromRequest;
use sqlx::types::Uuid;
use std::future::{Ready, ready};

pub struct StaffSession(Session);

pub struct CustomerSession(Session);

pub trait SessionState {
    const ID_KEY: &'static str = "id";

    fn value(&self) -> Session;
    fn from_session(session: Session) -> Self;

    fn renew(&self) {
        self.value().renew();
    }

    fn insert_sesh_id(&self, id: Uuid) -> Result<(), SessionInsertError> {
        self.value().insert(Self::ID_KEY, id)
    }

    fn get_sesh_id(&self) -> Result<Option<Uuid>, SessionGetError> {
        self.value().get(Self::ID_KEY)
    }

    fn log_out(&self) {
        self.value().purge();
    }
}

impl SessionState for StaffSession {
    fn value(&self) -> Session {
        self.0.clone()
    }

    fn from_session(session: Session) -> Self {
        StaffSession(session)
    }
}

impl SessionState for CustomerSession {
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
