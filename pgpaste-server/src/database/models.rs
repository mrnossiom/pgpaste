#![allow(clippy::missing_docs_in_private_items)]

//! `Diesel` models that represent database objects
// TODO: build a macro to reduce boilerplate and generate ids struct for each table with a `AsExpression` implementation

use super::schema::{self, pastes, public_keys};
use diesel::{
	deserialize::{self, FromSql},
	pg::{Pg, PgValue},
	serialize::{self, IsNull, Output, ToSql},
	AsChangeset, AsExpression, FromSqlRow, Identifiable, Insertable, Queryable, Selectable,
};
use std::{io::Write, time::SystemTime};

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq)]
#[diesel(sql_type = schema::sql_types::Visibility)]
pub enum Visibility {
	Public,
	Protected,
	Private,
}

impl ToSql<schema::sql_types::Visibility, Pg> for Visibility {
	fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
		match *self {
			Self::Public => out.write_all(b"public")?,
			Self::Protected => out.write_all(b"protected")?,
			Self::Private => out.write_all(b"private")?,
		}
		Ok(IsNull::No)
	}
}

impl FromSql<schema::sql_types::Visibility, Pg> for Visibility {
	fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
		match bytes.as_bytes() {
			b"public" => Ok(Self::Public),
			b"protected" => Ok(Self::Protected),
			b"private" => Ok(Self::Private),
			_ => Err("Unrecognized enum variant".into()),
		}
	}
}

impl From<pgpaste_api_types::Visibility> for Visibility {
	fn from(visibility: pgpaste_api_types::Visibility) -> Self {
		match visibility {
			pgpaste_api_types::Visibility::Public => Self::Public,
			pgpaste_api_types::Visibility::Protected => Self::Protected,
			pgpaste_api_types::Visibility::Private => Self::Private,
		}
	}
}

impl From<Visibility> for pgpaste_api_types::Visibility {
	fn from(visibility: Visibility) -> Self {
		match visibility {
			Visibility::Public => Self::Public,
			Visibility::Protected => Self::Protected,
			Visibility::Private => Self::Private,
		}
	}
}

/// Represent a single public key
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable)]
#[diesel(table_name = public_keys)]
pub(crate) struct PublicKey {
	pub(crate) id: i32,

	pub(crate) fingerprint: Vec<u8>,
	pub(crate) key: Vec<u8>,
}

/// Use to create a new [`PublicKey`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = public_keys)]
pub(crate) struct NewPublicKey<'a> {
	pub(crate) fingerprint: &'a [u8],
	pub(crate) key: &'a [u8],
}

/// Represent a single signed or encrypted paste
#[derive(Debug, PartialEq, Eq, Queryable, Identifiable, Selectable)]
#[diesel(table_name = pastes)]
pub(crate) struct Paste {
	pub(crate) id: i32,

	pub(crate) slug: String,
	pub(crate) visibility: Visibility,
	pub(crate) content: Vec<u8>,

	pub(crate) burn_at: SystemTime,
	pub(crate) created_at: SystemTime,
}

/// Use to create a new [`Paste`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = pastes)]
pub(crate) struct NewPaste<'a> {
	pub(crate) slug: &'a str,
	pub(crate) visibility: &'a Visibility,
	pub(crate) content: &'a [u8],
}
