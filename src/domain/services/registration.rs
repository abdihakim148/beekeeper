/// A trait representing the registration process.
use crate::ports::outputs::database::Table;
use crate::ports::outputs::database::Item;
use crate::domain::types::{User, Error, Id};
// use bson::oid::ObjectId;
use argon2::PasswordHasher;

use super::Password;

pub trait Registration: Sized + Item {
    type Id;
    type Error;
    /// Registers a new entity.
    ///
    /// # Returns
    ///
    /// * `Result<Self::Id>` - Returns the ID of the registered entity wrapped in a `Result`.
    async fn register<T: Table<Item = Self, Error: Into<Self::Error>>, H: PasswordHasher>(&self, table: &T, argon2: &H) -> Result<Self, Self::Error>;
}

impl Registration for User {
    type Id = Id;
    type Error = Error;
    async fn register<T: Table<Item = Self, Error: Into<Self::Error>>, H: PasswordHasher>(&self, table: &T, argon2: &H) -> Result<Self, Self::Error> {
        let id = self.id;
        let username = self.username.clone();
        let first_name = self.first_name.clone();
        let last_name = self.last_name.clone();
        let contact = self.contact.clone();
        let password = self.password.hash(argon2)?;
        let mut user = Self{id, username, first_name, last_name, contact, password};
        let result = table.create(&user).await;
        user.id = match result {
            Ok(id) => id,
            Err(err) => return Err(err.into())
        };
        user.password = Default::default();
        Ok(user)
    }
}
