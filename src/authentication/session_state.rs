use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::FromRequest;
use sqlx::types::Uuid;
use std::future::{Ready, ready};

pub struct TypedSession(Session);

impl TypedSession {
    const ID_KEY: &'static str = "id";

    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn insert_sesh_id(&self, id: Uuid) -> Result<(), SessionInsertError> {
        self.0.insert(Self::ID_KEY, id)
    }

    pub fn get_sesh_id(&self) -> Result<Option<Uuid>, SessionGetError> {
        self.0.get(Self::ID_KEY)
    }

    pub fn log_out(self) {
        self.0.purge();
    }
}

impl FromRequest for TypedSession {
    type Error = <Session as FromRequest>::Error;
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
