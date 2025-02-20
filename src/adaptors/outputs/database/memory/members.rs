//! Members collection implementation for the memory database
//! 
//! This module provides the implementation for storing and managing member records
//! in memory with thread-safe access and index management.

use crate::ports::outputs::database::{Item, CreateItem, GetItem, UpdateItem, DeleteItem, Map};
use crate::domain::types::{Member, Id, Key, Value, User, Organisation};
use std::collections::HashMap;
use std::sync::RwLock as Lock;
use super::error::Error;

/// Thread-safe, indexed storage for member records
/// 
/// # Indexes
/// - Primary index: (Organisation ID, User ID) -> Member record
/// - Secondary indexes:
///   * Organisation ID -> Vec<Member>
///   * User ID -> Vec<Member>
/// 
/// # Concurrency
/// Uses RwLock to ensure safe concurrent read and write operations
#[derive(Debug, Default)]
pub struct Members {
    /// Primary storage of members, keyed by (org_id, user_id)
    pub members: Lock<HashMap<<Member as Item>::PK, Member>>,
    
    /// Secondary index mapping organisation IDs to user IDs
    pub org_index: Lock<HashMap<<Organisation as Item>::PK, Vec<<User as Item>::PK>>>,
    
    /// Secondary index mapping user IDs to organisation IDs
    pub user_index: Lock<HashMap<<User as Item>::PK, Vec<<Organisation as Item>::PK>>>,
}

impl Members {
    /// Updates secondary indexes when a member's details change
    /// 
    /// # Arguments
    /// * `member`: The member being added or updated
    /// 
    /// # Behavior
    /// - Adds/updates member in organisation and user indexes
    pub fn update_indexes(&self, member: &Member) -> Result<(), Error> {
        // Update organisation index
        let mut org_index = self.org_index.write()?;
        org_index.entry(member.org_id)
            .or_insert_with(Vec::new)
            .retain(|&user_id| user_id != member.user_id);
        org_index.get_mut(&member.org_id)
            .unwrap()
            .push(member.user_id);

        // Update user index
        let mut user_index = self.user_index.write()?;
        user_index.entry(member.user_id)
            .or_insert_with(Vec::new)
            .retain(|&org_id| org_id != member.org_id);
        user_index.get_mut(&member.user_id)
            .unwrap()
            .push(member.org_id);

        Ok(())
    }

    /// Removes a member from secondary indexes
    /// 
    /// # Arguments
    /// * `member`: The member being removed
    pub fn remove_from_indexes(&self, member: &Member) -> Result<(), Error> {
        // Remove from organisation index
        let mut org_index = self.org_index.write()?;
        if let Some(org_members) = org_index.get_mut(&member.org_id) {
            org_members.retain(|&user_id| user_id != member.user_id);
        }

        // Remove from user index
        let mut user_index = self.user_index.write()?;
        if let Some(user_members) = user_index.get_mut(&member.user_id) {
            user_members.retain(|&org_id| org_id != member.org_id);
        }

        Ok(())
    }
}

impl CreateItem<Member> for Members {
    type Error = Error;
    
    async fn create_item(&self, member: Member) -> Result<Member, Self::Error> {
        // Check if member already exists
        if self.members.read()?.contains_key(&(member.org_id, member.user_id)) {
            return Err(Error::MemberAlreadyExists);
        }
        
        // Update indexes
        self.update_indexes(&member)?;
        
        // Store member
        self.members.write()?.insert((member.org_id, member.user_id), member.clone());
        
        Ok(member)
    }
}

impl GetItem<(Organisation, User), Member> for Members {
    type Error = Error;

    async fn get_item(&self, key: Key<&<(Organisation, User) as Item>::PK, &<(Organisation, User) as Item>::SK>) -> Result<Member, Self::Error> {
        let option = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => self.members.read()?.get(pk).cloned(),
            _ => None
        };

        if let Some(member) = option {
            return Ok(member)
        }
        Err(Error::ServiceNotFound)
    }
}

impl UpdateItem<(Organisation, User), Member> for Members {
    type Error = Error;
    type Update = Map;

    async fn update_item(&self, key: Key<&<(Organisation, User) as Item>::PK, &<(Organisation, User) as Item>::SK>, member: Member) -> Result<Member, Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            _ => return Err(Error::MemberNotFound),
        };

        // Remove old indexes
        if let Some(old_member) = self.members.read()?.get(&pk) {
            self.remove_from_indexes(old_member)?;
        }
        
        // Update indexes
        self.update_indexes(&member)?;
        
        // Store updated member
        self.members.write()?.insert(pk, member.clone());
        Ok(member)
    }

    /// Partially update a member's fields
    /// 
    /// # Arguments
    /// * `key`: The key to identify the member to update
    /// * `map`: A map of fields to update
    /// 
    /// # Returns
    /// The updated member or an error if the update fails
    /// 
    /// # Behavior
    /// - Allows updating title, owner status, and roles
    async fn patch_item(&self, key: Key<&<(Organisation, User) as Item>::PK, &<(Organisation, User) as Item>::SK>, map: Map) -> Result<Member, Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            _ => return Err(Error::MemberNotFound),
        };

        let mut members = self.members.write()?;
        let member = members.get_mut(&pk).ok_or(Error::MemberNotFound)?;

        // Update basic fields
        if let Some(value) = map.get("title") {
            member.title = value.clone().try_into()?;
        }
        if let Some(value) = map.get("owner") {
            member.owner = value.clone().try_into()?;
        }
        if let Some(value) = map.get("roles") {
            member.roles = value.clone().try_into()?;
        }

        // Remove old indexes and update with new member data
        self.remove_from_indexes(member)?;
        self.update_indexes(member)?;

        Ok(member.clone())
    }

    /// Delete specific fields from a member
    /// 
    /// # Arguments
    /// * `key`: The key to identify the member to update
    /// * `fields`: List of field names to delete
    /// 
    /// # Returns
    /// The updated member or an error if the deletion fails
    /// 
    /// # Behavior
    /// - Members do not support deleting individual fields
    async fn delete_fields(&self, _key: Key<&<(Organisation, User) as Item>::PK, &<(Organisation, User) as Item>::SK>, _fields: &[String]) -> Result<Member, Self::Error> {
        Err(Error::UnsupportedOperation)
    }
}

impl DeleteItem<Member> for Members {
    type Error = Error;
    
    async fn delete_item(&self, key: Key<&<Member as Item>::PK, &<Member as Item>::SK>) -> Result<(), Self::Error> {
        let pk = match key {
            Key::Pk(pk) | Key::Both((pk, _)) => *pk,
            _ => return Err(Error::MemberNotFound),
        };

        // Remove from indexes
        if let Some(member) = self.members.read()?.get(&pk) {
            self.remove_from_indexes(member)?;
        }

        // Remove member
        self.members.write()?.remove(&pk);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bson::oid::ObjectId;

    /// Helper function to create a test member
    fn create_test_member() -> Member {
        Member {
            org_id: Id(ObjectId::new()),
            user_id: Id(ObjectId::new()),
            title: "Test Member".to_string(),
            owner: false,
            roles: vec![Id(ObjectId::new())],
        }
    }

    #[tokio::test]
    async fn test_create_member() {
        let members = Members::default();
        let member = create_test_member();
        let result = members.create_item(member.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), member);
    }

    #[tokio::test]
    async fn test_create_duplicate_member() {
        let members = Members::default();
        let member1 = create_test_member();
        let mut member2 = member1.clone();
        
        let _ = members.create_item(member1).await;
        let result = members.create_item(member2).await;
        assert!(matches!(result, Err(Error::MemberAlreadyExists)));
    }

    #[tokio::test]
    async fn test_get_member_by_org() {
        let members = Members::default();
        let member = create_test_member();
        let _ = members.create_item(member.clone()).await;
        
        let result = members.get_item(Key::Pk(&(member.org_id, member.user_id))).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), member);
    }

    #[tokio::test]
    async fn test_get_member_by_user() {
        let members = Members::default();
        let member = create_test_member();
        let _ = members.create_item(member.clone()).await;

        let result = members.get_item(Key::Pk(&(member.org_id, member.user_id))).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), member);
    }

    #[tokio::test]
    async fn test_patch_member_title() {
        let members = Members::default();
        let member = create_test_member();
        let _ = members.create_item(member.clone()).await;
        
        let patch_map = HashMap::from([
            ("title".to_string(), Value::String("Updated Member Title".to_string()))
        ]);

        let result = members.patch_item(Key::Pk(&(member.org_id, member.user_id)), patch_map).await;
        assert!(result.is_ok(), "Patching member title should succeed");
        
        let updated_member = result.unwrap();
        assert_eq!(updated_member.title, "Updated Member Title", "Member title should be updated");
    }

    #[tokio::test]
    async fn test_patch_member_owner_status() {
        let members = Members::default();
        let member = create_test_member();
        let _ = members.create_item(member.clone()).await;
        
        let patch_map = HashMap::from([
            ("owner".to_string(), Value::Bool(true))
        ]);

        let result = members.patch_item(Key::Pk(&(member.org_id, member.user_id)), patch_map).await;
        assert!(result.is_ok(), "Patching member owner status should succeed");
        
        let updated_member = result.unwrap();
        assert!(updated_member.owner, "Member owner status should be updated");
    }

    #[tokio::test]
    async fn test_delete_member_fields_unsupported() {
        let members = Members::default();
        let member = create_test_member();
        let _ = members.create_item(member.clone()).await;
        
        let result = members.delete_fields(Key::Pk(&(member.org_id, member.user_id)), &["title".to_string()]).await;
        assert!(matches!(result, Err(Error::UnsupportedOperation)), 
                "Deleting fields should return UnsupportedOperation");
    }
}
