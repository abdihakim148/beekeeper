/// Module for database tables.
mod tables;
mod error;

use crate::ports::outputs::database::Database;
use crate::ports::outputs::database::Table;
use tokio::sync::OnceCell;
pub use tables::*;
pub use error::*;


pub static MEMORY: OnceCell<Memory> = OnceCell::const_new();

/// A struct representing an in-memory database.
/// A struct representing an in-memory database.
pub struct Memory {
    users: Users,
}

impl Database for Memory {
    type Users = Users;
    type Config = ();
    type Error = Error;

    async fn new(_config: ()) -> Result<Self, Self::Error> {
        let users = Users::new().await?;
        Ok(Memory { users })
    }

    async fn users<'a>(&'a self) -> Result<&'a Self::Users, Self::Error> {
        Ok(&self.users)
    }
}
