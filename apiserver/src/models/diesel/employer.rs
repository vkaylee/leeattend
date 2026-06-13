use crate::models::employer::Employer;
use crate::schema::employers;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    Queryable,
    Selectable,
    Insertable,
    Identifiable,
    AsChangeset,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = employers)]
pub struct EmployerEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub tax_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub legal_address: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub parent_id: Option<Uuid>,
    pub relationship_type: Option<String>,
    pub business_type: Option<String>,
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub incorporation_date: Option<chrono::NaiveDate>,
    pub country_code: Option<String>,
    pub currency_code: Option<String>,
}

impl From<EmployerEntity> for Employer {
    fn from(entity: EmployerEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            tax_code: entity.tax_code,
            legal_address: entity.legal_address,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            parent_id: entity.parent_id,
            relationship_type: entity.relationship_type,
            business_type: entity.business_type,
            registration_number: entity.registration_number,
            phone: entity.phone,
            email: entity.email,
            website: entity.website,
            incorporation_date: entity.incorporation_date,
            country_code: entity.country_code,
            currency_code: entity.currency_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employer_entity_from() {
        let id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let now = Utc::now();
        let entity = EmployerEntity {
            id,
            workspace_id: ws_id,
            name: "Test Employer".to_string(),
            tax_code: Some("123456".to_string()),
            legal_address: Some("123 Address Rd".to_string()),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            parent_id: None,
            relationship_type: None,
            business_type: Some("llc".to_string()),
            registration_number: Some("REG123".to_string()),
            phone: Some("+123456789".to_string()),
            email: Some("employer@test.com".to_string()),
            website: Some("https://test.com".to_string()),
            incorporation_date: Some(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            country_code: Some("VNM".to_string()),
            currency_code: Some("VND".to_string()),
        };

        let employer: Employer = entity.into();
        assert_eq!(employer.id, id);
        assert_eq!(employer.workspace_id, ws_id);
        assert_eq!(employer.name, "Test Employer");
        assert_eq!(employer.tax_code, Some("123456".to_string()));
        assert_eq!(employer.legal_address, Some("123 Address Rd".to_string()));
        assert_eq!(employer.created_at, now);
        assert_eq!(employer.updated_at, now);
        assert_eq!(employer.parent_id, None);
        assert_eq!(employer.relationship_type, None);
        assert_eq!(employer.business_type, Some("llc".to_string()));
        assert_eq!(employer.registration_number, Some("REG123".to_string()));
        assert_eq!(employer.phone, Some("+123456789".to_string()));
        assert_eq!(employer.email, Some("employer@test.com".to_string()));
        assert_eq!(employer.website, Some("https://test.com".to_string()));
        assert_eq!(
            employer.incorporation_date,
            Some(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())
        );
        assert_eq!(employer.country_code, Some("VNM".to_string()));
        assert_eq!(employer.currency_code, Some("VND".to_string()));
    }
}
