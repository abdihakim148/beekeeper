use crate::domain::types::Error;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, MapAccess, Visitor};
use std::fmt;

/// An enum representing the state of an email.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Email {
    New(String),
    Verified(String),
}

impl Email {
    /// Creates a new Email instance after validating the email address format.
    ///
    /// # Arguments
    ///
    /// * `email` - A string slice that holds the email to validate.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns `Ok(Self)` if the email is valid, `Err(Error)` otherwise.
    pub fn new(email: &str) -> Result<Self, Error> {
        if email.contains('@') && email.contains('.') {
            Ok(Email::New(email.to_string()))
        } else {
            Err(Error::InvalidEmail)
        }
    }
}

impl Serialize for Email {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Email::New(email) => serializer.serialize_str(email),
            Email::Verified(email) => {
                let mut state = serializer.serialize_struct("Email", 2)?;
                state.serialize_field("email", email)?;
                state.serialize_field("verified", &true)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EmailVisitor;

        impl<'de> Visitor<'de> for EmailVisitor {
            type Value = Email;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or a map with an email and verified status")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Email::new(value)
                    .map_err(|_| de::Error::custom(Error::InvalidEmail.to_string()))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut email = None;
                let mut verified = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "email" => {
                            if email.is_some() {
                                return Err(de::Error::duplicate_field("email"));
                            }
                            email = Some(map.next_value()?);
                        }
                        "verified" => {
                            if verified.is_some() {
                                return Err(de::Error::duplicate_field("verified"));
                            }
                            verified = Some(map.next_value()?);
                        }
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let email: String = email.ok_or_else(|| de::Error::missing_field("email"))?;
                let verified: bool = verified.unwrap_or(false);
                if verified {
                    Ok(Email::Verified(email))
                } else {
                    Ok(Email::New(email))
                }
            }
        }

        deserializer.deserialize_any(EmailVisitor)
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_email_new_valid() {
        let email = "test@example.com";
        let result = Email::new(email);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Email::New(email.to_string()));
    }

    #[test]
    fn test_deserialize_invalid_email() {
        let data = "\"invalid-email\"";
        let result: Result<Email, _> = serde_json::from_str(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_email_new_invalid() {
        let email = "invalid-email";
        let result = Email::new(email);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::InvalidEmail);
    }

    #[test]
    fn test_serialize_new_email() {
        let email = Email::New("test@example.com".to_string());
        let serialized = serde_json::to_string(&email).unwrap();
        assert_eq!(serialized, "\"test@example.com\"");
    }

    #[test]
    fn test_serialize_verified_email() {
        let email = Email::Verified("verified@example.com".to_string());
        let serialized = serde_json::to_string(&email).unwrap();
        assert_eq!(serialized, "{\"email\":\"verified@example.com\",\"verified\":true}");
    }

    #[test]
    fn test_deserialize_new_email() {
        let data = "\"test@example.com\"";
        let email: Email = serde_json::from_str(data).unwrap();
        assert_eq!(email, Email::New("test@example.com".to_string()));
    }

    #[test]
    fn test_deserialize_verified_email() {
        let data = "{\"email\":\"verified@example.com\",\"verified\":true}";
        let email: Email = serde_json::from_str(data).unwrap();
        assert_eq!(email, Email::Verified("verified@example.com".to_string()));
    }
}