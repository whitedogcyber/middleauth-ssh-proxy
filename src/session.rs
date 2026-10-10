use std::error::Error;
use crate::token_storage::{TokenSession, TokenStorage};

pub struct Session {}

impl Session {
    pub fn get_session() -> Result<Option<TokenSession>, Box<dyn Error>> {
        match TokenStorage::get_session()? {
            Some(session) => {
                if session.is_expired(None) {
                    Ok(Some(session.refresh()?))
                }
                else {
                    Ok(Some(session))
                }
            },
            None => Ok(None)
        }
    }

    pub async fn get_session_async() -> Result<Option<TokenSession>, Box<dyn Error>> {
        match TokenStorage::get_session()? {
            Some(session) => {
                if session.is_expired(None) {
                    Ok(Some(session.refresh_async().await?))
                }
                else {
                    Ok(Some(session))
                }
            },
            None => Ok(None)
        }
    }
}