// main.rs

// *************************************************************************
// * Copyright (C) 2017-2018 Daniel Mueller (deso@posteo.net)              *
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
use std::path;
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


fn normalize(string: &str) -> String {
  let mut s = string.to_string();
  while s.contains("  ") {
    s = s.replace("  ", " ");
  }
  s
}

fn handle<F>(mut cursor: sqlite::Cursor, callback: &mut F) -> Result<()>
where
  F: FnMut(&str, &str, &str) -> Result<()>,
{
  while let Some(row) = cursor.next()? {
    let english = row[0].as_string().ok_or(Error::Error(format!(
      "Invalid first column in result: {:?}",
      row
    )))?;
    let german = row[1].as_string().ok_or(Error::Error(format!(
      "Invalid second column in result: {:?}",
      row
    )))?;
    let type_ = row[2].as_string().ok_or(Error::Error(format!(
      "Invalid third column in result: {:?}",
      row
    )))?;
    callback(&normalize(english), &normalize(german), type_)?;
  }
  Ok(())
}

fn open(db: &path::Path) -> Result<sqlite::Connection> {
  // Note that sqlite::open by default creates the database if it does
  // not exist. That is not a desired behavior. So we catch cases where
  // the database does not exist in advance.
  if !db.exists() {
    let path = db.to_str().ok_or(
      Error::Error("No database given".to_string()),
    )?;
    Err(Error::Error(format!("Database {} not found", path).to_string()))
  } else {
    let connection = sqlite::open(db)?;
    Ok(connection)
  }
}

fn translate<F>(db: &path::Path, to_translate: &str, mut callback: F) -> Result<()>
where
  F: FnMut(&str, &str, &str) -> Result<()>,
{
  let connection = open(db)?;
  // Note that for some reason some terms in the database do not have a
  // proper type associated with them. We make this fact a little more
  // explicit by replacing the empty string. Note that it is important
  // to properly handle this problem at the level of SQL. We sort by the
  // type column and if we perform the replacement afterwards we mess up
  // the order because the empty string '' is sorted before all other
  // strings.
  let select = format!(
    "SELECT {eng},{ger}, \
       CASE {typ} WHEN '' \
         THEN 'unknown' \
         ELSE entry_type \
       END AS __type__, \
       {use} \
     FROM {tbl}",
    eng = ENGLISH_COL, ger = GERMAN_COL,
    typ = TYPE_COL, tbl = SEARCH_TBL, use = USAGE_COL,
  );
  // Note that the database contains some elements with strings
  // containing multiple white spaces in succession. As of now we only
  // support two spaces and will merge them into a single one. Do note
  // though that the entire (current) data set was checked and it was
  // found that only square braces ever appear with two spaces in front
  // of them.
  let where1 = format!(
    "WHERE {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           ({eng} LIKE ? AND __type__='verb') OR \
           ({eng} LIKE ? AND __type__='verb')",
    eng = ENGLISH_COL,
  );
  let where2 = format!(
    "WHERE {eng} LIKE ? OR \
           {eng} LIKE ? OR \
           {eng} LIKE ?",
    eng = ENGLISH_COL,
  );
  // We order by type first and then by the number of uses. The reason
  // is that we first want to print all the translations for a
  // particular type sorted by the number of uses before moving on to
  // the next type.
  let order = format!(
    "ORDER BY __type__ ASC, \
             {use} DESC, \
             {eng} ASC",
    eng = ENGLISH_COL, use = USAGE_COL,
  );

  let query =
    format!(
    "{select} {where1} \
     UNION \
     {select} {where2} \
     {order}",
    select = select, where1 = where1, where2 = where2, order = order,
  );

  let mut cursor = connection.prepare(query)?.cursor();
  cursor.bind(&[
    vec![sqlite::Value::String(to_translate.to_string())],
    include!("permutations.in"),
    vec![
      sqlite::Value::String(
        "to ".to_string() + to_translate
      ),
      sqlite::Value::String(
        "to ".to_string() + to_translate + " %"
      ),
      sqlite::Value::String(
        to_translate.to_string() + " %"
      ),
      sqlite::Value::String(
        "% ".to_string() + to_translate
      ),
      sqlite::Value::String(
        "% ".to_string() + to_translate + " %"
      ),
    ],
  ]
   .concat())?;

  handle(cursor, &mut callback)
}

fn print_usage(program: &str, opts: getopts::Options) {
  let usage = format!("Usage: {} <database> <word> [options]", program);
  print!("{}", opts.usage(&usage));
}

fn run() -> i32 {
  let argv: Vec<String> = env::args().collect();
  let program = argv[0].clone();
  let mut opts = getopts::Options::new();
  opts.optopt("d", "", "the database to use", "DATABASE");

  let matches = match opts.parse(&argv[1..]) {
      Ok(m) => { m }
      Err(f) => { panic!(f.to_string()) }
  };
  let db = matches.opt_str("d").unwrap();
  let db = path::Path::new(&db);
  if matches.free.is_empty() {
    print_usage(&program, opts);
    return 1;
  };

  let callback = |english: &str, german: &str, type_: &str| {
    println!("{} ({}): {}", english, type_, german);
    Ok(())
  };

  // We treat all arguments past the database path itself as words to
  // search for (in that order, with a single space in between them).
  match translate(&db, &matches.free.join(" "), callback) {
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


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fail_db_not_found() {
    let db = path::Path::new("./test/does_not_exist.db");
    let callback = |_: &str, _: &str, _: &str| {
      assert!(false);
      Err(Error::Error("unreachable".to_string()))
    };

    let err = translate(db, &"", callback).unwrap_err();
    match err {
      Error::Error(x) => assert_eq!(x, "Database ./test/does_not_exist.db not found"),
      _ => panic!("Unexpected error: {}", err),
    }
  }

  #[test]
  fn no_results() {
    let db = path::Path::new("./test/test.db");
    let callback = |_: &str, _: &str, _: &str| {
      assert!(false);
      Err(Error::Error("unreachable".to_string()))
    };

    // We attempt translation of a word that has no translations. We
    // expect no errors.
    translate(db, &"awordthatdoesnotexist", callback).unwrap();
  }

  fn collect_translations(to_translate: &str) -> Vec<(String, String, String)> {
    let mut found = Vec::new();
    let db = path::Path::new("./test/test.db");
    {
      let callback = |english: &str, german: &str, type_: &str| {
        found.push((english.to_string(), type_.to_string(), german.to_string()));
        Ok(())
      };

      translate(db, to_translate, callback).unwrap();
    }
    found
  }

  #[test]
  fn inject_malicious_sql() {
    // By injecting a condition that is always true we would effectively
    // dump the entire table's contents, if the code were prone to SQL
    // injection.
    let code = format!("' OR 1=1 OR {eng}='", eng = ENGLISH_COL);
    let found = collect_translations(&code);
    assert_eq!(found, vec![]);
  }

  #[test]
  fn translate_nauseating() {
    let found = collect_translations(&"nauseating");
    assert_eq!(
      found,
      vec![
        ("nauseating".to_string(), "adj".to_string(), "ekelerregend".to_string()),
        ("nauseating".to_string(), "adj".to_string(), "widerlich".to_string()),
      ]
    );
  }

  #[test]
  fn translate_surefire() {
    let found = collect_translations(&"surefire");
    assert_eq!(
      found,
      vec![
        ("surefire [coll.]".to_string(), "adj".to_string(), "todsicher [ugs.]".to_string()),
      ]
    );
  }

  #[test]
  fn translate_dorky() {
    let found = collect_translations(&"dorky");
    assert_eq!(
      found,
      vec![
        ("dorky [coll.]".to_string(), "adj".to_string(), "bekloppt [ugs.]".to_string()),
        ("dorky [coll.]".to_string(), "adj".to_string(), "idiotisch".to_string()),
        ("dorky [coll.]".to_string(), "adj".to_string(), "deppert [österr.] [südd.]".to_string()),
      ]
    );
  }

  #[test]
  fn translate_subjugate() {
    let found = collect_translations(&"subjugate");
    assert_eq!(
      found,
      vec![
        ("to subjugate".to_string(), "verb".to_string(), "unterwerfen".to_string()),
        ("to subjugate".to_string(), "verb".to_string(), "bezwingen".to_string()),
        ("to subjugate".to_string(), "verb".to_string(), "unterjochen".to_string()),
        (
          "to subjugate sb./sth.".to_string(),
          "verb".to_string(),
          "jdn./etw. knechten [geh.] [pej.] [unterwerfen]".to_string()
        ),
      ]
    );
  }

  #[test]
  fn translate_love() {
    let found = collect_translations(&"love");
    assert_eq!(
      found,
      vec![
        ("love".to_string(), "noun".to_string(), "Liebe {f}".to_string()),
        ("love".to_string(), "unknown".to_string(), "null [beim Tennis]".to_string()),
      ]
    );
  }

  #[test]
  fn translate_christmas() {
    let found = collect_translations(&"christmas");
    assert_eq!(
      found,
      vec![
        ("Christmas".to_string(), "noun".to_string(), "Weihnachten {n}".to_string()),
      ]
    );
  }

  #[test]
  fn translate_wherewithals() {
    let found = collect_translations(&"wherewithals");
    assert_eq!(
      found,
      vec![
        ("wherewithals {pl}".to_string(), "noun".to_string(), "Nötiges {n}".to_string()),
      ]
    );
  }

  #[test]
  fn translate_statistics() {
    let found = collect_translations(&"statistics");
    assert_eq!(
      found,
      vec![
        (
          "statistics {pl} [science that collects and interprets numerical data] [treated as sg.] \
           <stats>"
          .to_string(),
          "noun".to_string(),
          "Statistik {f}".to_string()
        ),
        ("statistics".to_string(), "noun".to_string(), "Statistiken {pl}".to_string()),
      ]
    );
  }

  #[test]
  fn translate_contents() {
    let found = collect_translations(&"contents");
    assert_eq!(
      found,
      vec![
        ("contents {pl} <cont.>".to_string(), "noun".to_string(), "Inhalt {m} <Inh.>".to_string()),
      ]
    );
  }

  #[test]
  fn translate_sulfur() {
    let found = collect_translations(&"sulfur");
    assert_eq!(
      found,
      vec![
        ("sulfur <S> [Am.]".to_string(), "noun".to_string(), "Schwefel {m} <S>".to_string()),
      ]
    );
  }

  #[test]
  fn translate_poor() {
    let found = collect_translations(&"poor");
    assert_eq!(
      found,
      vec![
        (
          "the poor {pl}".to_string(),
          "noun".to_string(),
          "Arme {pl} [arme Leute als Klasse]".to_string()
        ),
      ]
    );
  }
}
