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
use std::io::Write;

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
			Visibility::Public => out.write_all(b"public")?,
			Visibility::Protected => out.write_all(b"protected")?,
			Visibility::Private => out.write_all(b"private")?,
		}
		Ok(IsNull::No)
	}
}

impl FromSql<schema::sql_types::Visibility, Pg> for Visibility {
	fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
		match bytes.as_bytes() {
			b"public" => Ok(Visibility::Public),
			b"protected" => Ok(Visibility::Protected),
			b"private" => Ok(Visibility::Private),
			_ => Err("Unrecognized enum variant".into()),
		}
	}
}

impl From<pgpaste_api_types::Visibility> for Visibility {
	fn from(visibility: pgpaste_api_types::Visibility) -> Self {
		match visibility {
			pgpaste_api_types::Visibility::Public => Visibility::Public,
			pgpaste_api_types::Visibility::Protected => Visibility::Protected,
			pgpaste_api_types::Visibility::Private => Visibility::Private,
		}
	}
}

impl From<Visibility> for pgpaste_api_types::Visibility {
	fn from(visibility: Visibility) -> Self {
		match visibility {
			Visibility::Public => pgpaste_api_types::Visibility::Public,
			Visibility::Protected => pgpaste_api_types::Visibility::Protected,
			Visibility::Private => pgpaste_api_types::Visibility::Private,
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
}

/// Use to create a new [`Paste`]
#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = pastes)]
pub(crate) struct NewPaste<'a> {
	pub(crate) slug: &'a str,
	pub(crate) visibility: &'a Visibility,
	pub(crate) content: &'a [u8],
}
