use crate::models::schema::users;
use bitcoin::secp256k1::PublicKey;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(
    QueryableByName, Queryable, AsChangeset, Serialize, Deserialize, Debug, Clone, PartialEq,
)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub pubkey: String,
    pub name: String,
    pub disabled_zaps: bool,
}

impl User {
    pub fn pubkey(&self) -> PublicKey {
        PublicKey::from_str(&self.pubkey).expect("invalid pubkey")
    }

    pub fn get_users(conn: &mut PgConnection) -> anyhow::Result<Vec<User>> {
        Ok(users::table.load::<Self>(conn)?)
    }

    pub fn get_by_id(conn: &mut PgConnection, user_id: i32) -> anyhow::Result<Option<User>> {
        Ok(users::table
            .filter(users::id.eq(user_id))
            .first::<User>(conn)
            .optional()?)
    }

    pub fn get_by_name(conn: &mut PgConnection, name: &str) -> anyhow::Result<Option<User>> {
        Ok(users::table
            .filter(users::name.eq(name))
            .first::<User>(conn)
            .optional()?)
    }

    pub fn check_available_name(conn: &mut PgConnection, name: String) -> anyhow::Result<bool> {
        Ok(users::table
            .filter(users::name.eq(name))
            .count()
            .get_result::<i64>(conn)?
            == 0)
    }

    pub fn get_by_pubkey(conn: &mut PgConnection, pubkey: String) -> anyhow::Result<Option<User>> {
        Ok(users::table
            .filter(users::pubkey.eq(pubkey))
            .first::<User>(conn)
            .optional()?)
    }

    pub fn disable_zaps(&self, conn: &mut PgConnection) -> anyhow::Result<()> {
        diesel::update(users::table)
            .filter(users::name.eq(&self.name))
            .set((users::disabled_zaps.eq(true),))
            .execute(conn)?;

        Ok(())
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub pubkey: String,
    pub name: String,
}

impl NewUser {
    pub fn insert(&self, conn: &mut PgConnection) -> anyhow::Result<User> {
        diesel::insert_into(users::table)
            .values(self)
            .get_result::<User>(conn)
            .map_err(|e| e.into())
    }
}
