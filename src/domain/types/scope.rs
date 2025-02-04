#[cfg(feature = "http")]
use actix_web::{Responder, web::Json, http::{Method, StatusCode}};
use crate::ports::outputs::database::Item;
use serde::{Serialize, Deserialize};
use super::{Id, Permission, Error};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scope{
    /// This would be the Id of the service where the resource belongs to
    pub id: Id,
    /// This would be the name of the resource.
    pub name: String,
    /// This would be the permission of the scope.
    pub permission: Permission
}


impl From<Scope> for String {
    fn from(scope: Scope) -> Self {
        format!("{}:{}:{}", scope.id.to_hex(), scope.name, scope.permission)
    }
}


impl FromStr for Scope {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split(':').collect::<Vec<&str>>();
        if splits.len() != 3 {
            return  Err(Error::conversion_error(Some("invalid scope")));
        }
        let (id, name, permission) = (splits[0], splits[1], splits[2]);
        let (id, name, permission) = (id.parse()?, name.to_string(), permission.parse()?);
        Ok(Scope{id, name, permission})
    }
}


#[cfg(feature = "http")]
impl Responder for Scope {
    type Body = <Json<Self> as Responder>::Body;
    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match req.method() {
            &Method::POST => {
                let mut res = Json(self).respond_to(req);
                *res.status_mut() = StatusCode::CREATED;
                res
            },
            &Method::GET => Json(self).respond_to(req),
            _ => Json(self).respond_to(req)
        }
    }
}


impl Item for Scope {
    const NAME: &'static str = "scope";
    const FIELDS: &'static [&'static str] = &["id", "name", "permission"];
    /// This is the id of the Owner of the scope.
    type PK = Id;
    /// This is the name of the scope.
    type SK = String;
}