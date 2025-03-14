use super::super::types::{Token, User, Paseto, Key, Audience, Error as DomainError};
use crate::ports::{Error, outputs::database::{Item, CreateItem, GetItem}};
use argon2::{PasswordHasher, PasswordVerifier};
use super::{Password, Paseto as PasetoTrait};


pub trait Authentication: Sized + Item {
    type Error;
    type QueryKey;
    async fn register<DB: CreateItem<Self>, H: PasswordHasher>(self, db: &DB, hasher: &H, paseto: &Paseto, issuer: String, audience: Audience) -> Result<Token, Self::Error>;
    async fn authenticate<DB: GetItem<Self>, V: PasswordVerifier>(query_key: &Self::QueryKey, password: &str, db: &DB, verifier: &V, paseto: &Paseto, issuer: String, audience: Audience) -> Result<Token, Self::Error>;
    async fn authorize(token: &str, paseto: &Paseto) -> Result<<Self as Item>::PK, Self::Error>;
}



impl Authentication for User {
    type Error = Error;
    type QueryKey = Self::SK;

    async fn register<DB: CreateItem<Self>, H: PasswordHasher>(mut self, db: &DB, hasher: &H, paseto: &Paseto, issuer: String, audience: Audience) -> Result<Token, Self::Error> {
        self.password = self.password.hash(hasher)?;
        let mut user = db.create_item(self).await?;
        let keys = &paseto.keys;
        let ttl = paseto.ttl;
        let token = user.token(issuer, audience, ttl).try_sign(keys)?;
        Ok(token)
    }


    async fn authenticate<DB: GetItem<Self>, V: PasswordVerifier>(contact: &Self::QueryKey, password: &str, db: &DB, verifier: &V, paseto: &Paseto, issuer: String, audience: Audience) -> Result<Token, Self::Error> {
        let key = Key::Sk(contact);
        let user = db.get_item(key).await?;
        let hash = &user.password;
        password.verify(hash, verifier)?;
        let keys = &paseto.keys;
        let ttl = paseto.ttl;
        let token = user.token(issuer, audience, ttl).try_sign(keys)?;
        Ok(token)
    }

    async fn authorize(signature: &str, paseto: &Paseto) -> Result<<Self as Item>::PK, Self::Error> {
        let keys = &paseto.keys;
        let token = Token::try_verify(signature, keys)?;
        if token.expired() {
            Err(DomainError::TokenExpired)?
        }
        Ok(token.subject)
    }
}