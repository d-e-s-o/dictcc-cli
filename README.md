dictcc-cli
==========

- [Changelog](CHANGELOG.md)

Purpose
-------

**dictcc-cli** is a command line interface to
[dict.cc](https://www.dict.cc/). As such, it provides the capability to
translate words and phrases between languages. However, instead of
interacting with the online service it relies on its database for the
translation. This database has to be [retrieved
manually](#language-database).


Usage
-----

Being intended to be simple and fast to use, translating a term using
**dictcc-cli** is as simple as providing it as an argument to the
program:
```bash
$ dictcc-cli dictcc-lp1.db dorky
> dorky [coll.] (adj): bekloppt [ugs.]
> dorky [coll.] (adj): idiotisch
> dorky [coll.] (adj): deppert [österr.] [südd.]
```


Installation
------------

#### From Source
In order to compile the program the `sqlite` crate needs to be available
which allows access to the dict.cc database. This crate and its dependencies
are contained in the form of subrepos in compatible and tested versions. Cargo
is required to build the program.

The build is as simple as running:
```bash
$ cargo build --release
```

#### Language Database
The database containing the translations has to be retrieved separately
as there are various languages and sizes available. One possible way is
to install the dict.cc app on an Android phone, install the desired
language pack, and then copy the file `cc.dict.dictcc/dictcc-lp1.db` to
the device running **dictcc-cli**.
