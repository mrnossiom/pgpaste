// Return types tends to be complex in here
#![allow(clippy::type_complexity)]

//! Small bits of `Diesel` queries to reuse across the project

use super::{
	models::{NewPaste, Paste},
	prelude::*,
	schema::pastes,
};
use diesel::{
	dsl::insert_into,
	helper_types::{Eq, Filter},
	query_builder::InsertStatement,
};

impl Paste {
	/// Select a paste from his `slug`
	#[inline]
	pub(crate) fn with_slug(slug: &str) -> Filter<pastes::table, Eq<pastes::slug, &str>> {
		pastes::table.filter(pastes::slug.eq(slug))
	}
}

impl<'a> NewPaste<'a> {
	/// Prepare a [`NewPaste`] insert
	#[inline]
	pub(crate) fn insert(
		&'a self,
	) -> InsertStatement<pastes::table, <&'a NewPaste<'a> as Insertable<pastes::table>>::Values> {
		insert_into(pastes::table).values(self)
	}
}
