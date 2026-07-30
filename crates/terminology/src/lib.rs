//! CDISC terminology deserialisation.
//!
//! Reads an SDTM or ADaM terminology workbook (`.xls`/`.xlsx`) and produces
//! a [`TerminologyVersion`] containing all the [`CodeList`]s and their
//! [`CodeItem`]s.

mod loader;
mod model;