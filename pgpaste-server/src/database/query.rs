// Return types tends to be complex in here
#![allow(clippy::type_complexity)]

//! Small bits of `Diesel` queries to reuse across the project

use super::{
	models::{NewPaste, NewPublicKey, Paste, PublicKey},
	prelude::*,
	schema::{pastes, public_keys},
};
use diesel::{
	dsl::{insert_into, now},
	helper_types::{Eq, Filter, Gt, LtEq},
	query_builder::InsertStatement,
};
use sequoia_openpgp::Fingerprint;

impl PublicKey {
	/// Select a public key from his `fingerprint`
	#[inline]
	pub(crate) fn with_fingerprint(
		fingerprint: &Fingerprint,
	) -> Filter<public_keys::table, Eq<public_keys::fingerprint, &[u8]>> {
		public_keys::table.filter(public_keys::fingerprint.eq(fingerprint.as_bytes()))
	}
}

impl<'a> NewPublicKey<'a> {
	/// Prepare a [`NewPublicKey`] insert
	#[inline]
	pub(crate) fn insert(
		&'a self,
	) -> InsertStatement<
		public_keys::table,
		<&'a NewPublicKey<'a> as Insertable<public_keys::table>>::Values,
	> {
		insert_into(public_keys::table).values(self)
	}
}

impl Paste {
	/// Select a paste from his `slug`
	#[inline]
	pub(crate) fn all_valid() -> Filter<pastes::table, Gt<pastes::burn_at, now>> {
		pastes::table.filter(pastes::burn_at.gt(now))
	}

	/// Select a paste from his `slug`
	#[inline]
	pub(crate) fn all_burnt() -> Filter<pastes::table, LtEq<pastes::burn_at, now>> {
		pastes::table.filter(pastes::burn_at.le(now))
	}

	/// Select a paste from his `slug`
	#[inline]
	pub(crate) fn with_slug(
		slug: &str,
	) -> Filter<Filter<pastes::table, Gt<pastes::burn_at, now>>, Eq<pastes::slug, &str>> {
		Self::all_valid().filter(pastes::slug.eq(slug))
	}

	/// Return the number of pastes associated with this public key
	#[inline]
	pub(crate) fn all_of_public_key(
		public_key_id: i32,
	) -> Filter<Filter<pastes::table, Gt<pastes::burn_at, now>>, Eq<pastes::public_key_id, i32>> {
		Self::all_valid().filter(pastes::public_key_id.eq(public_key_id))
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
