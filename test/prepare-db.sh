#!/bin/bash

# *************************************************************************
# * Copyright (C) 2017-2018 Daniel Mueller (deso@posteo.net)              *
# *                                                                       *
# * This program is free software: you can redistribute it and/or modify  *
# * it under the terms of the GNU General Public License as published by  *
# * the Free Software Foundation, either version 3 of the License, or     *
# * (at your option) any later version.                                   *
# *                                                                       *
# * This program is distributed in the hope that it will be useful,       *
# * but WITHOUT ANY WARRANTY; without even the implied warranty of        *
# * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the         *
# * GNU General Public License for more details.                          *
# *                                                                       *
# * You should have received a copy of the GNU General Public License     *
# * along with this program.  If not, see <http://www.gnu.org/licenses/>. *
# *************************************************************************

# We do not touch the original.
cp dictcc-lp1.db test.db
sqlite3 test.db < <(cat <<EOF
DELETE FROM main_ft WHERE NOT (
  term2 LIKE "anger" OR
  term2 LIKE "christmas" OR
  term2 LIKE "contents {%} <%>" OR
  term2 LIKE "dorky [%]" OR
  term2 LIKE "dorky  [%]" OR
  term2 LIKE "love" OR
  term2 LIKE "nauseating" OR
  term2 LIKE "speciation" OR
  term2 LIKE "%statistics%" OR
  term2 LIKE "sulfur <%> [%]" OR
  term2 LIKE "surefire [%]" OR
  term2 LIKE "the poor %" OR
  term2 LIKE "worse" OR
  term2 LIKE "wherewithals {%}" OR
  (term2 LIKE 'to subjugate' AND entry_type='verb') OR \
  (term2 LIKE 'to subjugate %' AND entry_type='verb')
);
DROP INDEX sw_term4search;
DROP TABLE subjects;
DROP TABLE main_ft_segments;
DROP TABLE main_ft_segdir;
DROP TABLE singlewords;
/* Check that a bunch of words are still in there. */
SELECT * from main_ft;
/* Check that our required query still works. */
SELECT * from main_ft where term2=="nauseating";
EOF
)
