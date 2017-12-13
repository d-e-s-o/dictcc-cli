// main.rs

// *************************************************************************
// * Copyright (C) 2017 Daniel Mueller (deso@posteo.net)                   *
// *                                                                       *
// * This program is free software: you can redistribute it and/or modify  *
// * it under the terms of the GNU General Public License as published by  *
// * the Free Software Foundation, either version 3 of the License, or     *
// * (at your option) any later version.                                   *
// *                                                                       *
// * This program is distributed in the hope that it will be useful,       *
// * but WITHOUT ANY WARRANTY; without even the implied warranty of        *
// * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the         *
// * GNU General Public License for more details.                          *
// *                                                                       *
// * You should have received a copy of the GNU General Public License     *
// * along with this program.  If not, see <http://www.gnu.org/licenses/>. *
// *************************************************************************

#![deny(missing_docs)]

//! dictcc-cli is a command line interface to translating between
//! languages by means of the offline data from dict.cc.

extern crate sqlite;

use std::env;
use std::fmt;
use std::process;
use std::result;

#[derive(Debug)]
/// Internally used error comprising the various different error types.
pub enum Error {
  /// An Sqlite error reported by the sqlite crate.
  SqlError(sqlite::Error),
  /// A custom error in the form of a string.
  Error(String),
}

impl From<sqlite::Error> for Error {
  fn from(e: sqlite::Error) -> Error {
    return Error::SqlError(e);
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *&self {
      &Error::SqlError(ref e) => return write!(f, "SQL error: {}", e),
      &Error::Error(ref e) => return write!(f, "Error: {}", e),
    }
  }
}

type Result<T> = result::Result<T, Error>;

// CREATE VIRTUAL TABLE "main_ft" using
//   fts3("id" INTEGER PRIMARY KEY NOT NULL,
//        "term1" VARCHAR,
//        "term2" VARCHAR,
//        "sort1" INTEGER,
//        "sort2" INTEGER,
//        "subj_ids" VARCHAR,
//        "entry_type" VARCHAR,
//        "vt_usage" INTEGER);
const SEARCH_TBL: &str = "main_ft";
const GERMAN_COL: &str = "term1";
const ENGLISH_COL: &str = "term2";
const TYPE_COL: &str = "entry_type";
const USAGE_COL: &str = "vt_usage";


fn translate(to_translate: &str) -> Result<()> {
  let connection = sqlite::open("./data/dictcc-lp1-2017-12-11_small.db")?;
  // We order by type first and then by the number of uses. The reason
  // is that we first want to print all the translations for a
  // particular type sorted by the number of uses before moving on to
  // the next type.
  // TODO: We need to escape our input before passing it to SQL or use a
  //       prepared statement or something of this sort to mitigate SQL
  //       injection problems.
  let query = format!(
    "SELECT {ger},{typ} FROM {tbl} \
     WHERE {eng}='{trans}' \
     ORDER BY {typ} ASC, \
              {use} DESC;",
    ger = GERMAN_COL, typ = TYPE_COL, tbl = SEARCH_TBL, eng = ENGLISH_COL,
    trans = to_translate, use = USAGE_COL,
  );

  let mut cursor = connection.prepare(query)?.cursor();

  while let Some(row) = cursor.next()? {
    let german = row[0].as_string().ok_or(Error::Error(format!(
      "Invalid first column in result: {:?}",
      row
    )))?;
    let type_ = row[1].as_string().ok_or(Error::Error(format!(
      "Invalid second column in result: {:?}",
      row
    )))?;
    println!("{} ({})", german, type_);
  }
  Ok(())
}

fn run() -> i32 {
  let argv: Vec<String> = env::args().collect();
  if argv.len() != 2 {
    eprintln!("Usage: {} <word>", argv[0]);
    return 1;
  }

  match translate(&argv[1]) {
    Ok(_) => 0,
    Err(e) => {
      eprintln!("{}", e);
      1
    },
  }
}

fn main() {
  process::exit(run());
}
