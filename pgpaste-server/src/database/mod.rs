//! Models and triggers related to database management

use diesel::{Connection, PgConnection};
use diesel_async::{
	AsyncPgConnection,
	pooled_connection::deadpool::{Object, Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use eyre::eyre;
use secrecy::ExposeSecret;

use crate::Config;

pub(crate) mod models;
pub(crate) mod query;
/// The automatically generated schema by `Diesel`
#[allow(clippy::wildcard_imports, clippy::missing_docs_in_private_items)]
#[rustfmt::skip]
pub(crate) mod schema;

#[allow(dead_code)]
/// The type alias for a Postgres connection pool
pub(crate) type DatabasePool = Pool<AsyncPgConnection>;
#[allow(dead_code)]
/// The type alias for a Postgres connection handle
pub(crate) type DatabasePooledConnection = Object<AsyncPgConnection>;

/// The migrations to apply to the database
pub(crate) const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Applies the migrations to the database
pub(crate) fn run_migrations(config: &Config) -> eyre::Result<()> {
	let mut connection = PgConnection::establish(config.database_url.expose_secret())?;

	let migrations_applied = connection
		.run_pending_migrations(MIGRATIONS)
		.map_err(|e| eyre!("Could not run migrations {}", e))?;

	tracing::debug!(migrations = ?migrations_applied, "Applied migrations");

	Ok(())
}

#[allow(unused_imports)]
/// Our own prelude for database related modules
pub(crate) mod prelude {
	pub(crate) use diesel::dsl as db_dsl;
	pub(crate) use diesel::prelude::{
		AppearsOnTable, AsChangeset, BelongingToDsl, BoolExpressionMethods, BoxableExpression,
		Column, CombineDsl, Connection, DecoratableTarget, EscapeExpressionMethods, Expression,
		ExpressionMethods, GroupedBy, Identifiable, Insertable, IntoSql, JoinOnDsl, JoinTo,
		NullableExpressionMethods, OptionalExtension, QueryDsl, QuerySource, Queryable,
		QueryableByName, Selectable, SelectableExpression, SelectableHelper, Table,
		TextExpressionMethods,
	};
	pub(crate) use diesel::result::Error as DieselError;
	pub(crate) use diesel_async::{RunQueryDsl, SaveChangesDsl, UpdateAndFetchResults};
}
