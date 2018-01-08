# permutations.py

# *************************************************************************
# * Copyright (C) 2018 Daniel Mueller (deso@posteo.net)                   *
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

from itertools import (
  chain,
  permutations,
)
from sys import (
  argv,
  exit,
  stderr,
)
from textwrap import (
  dedent,
)

def main(args):
  if len(args) != 2:
    print("Usage: %s <out-file>" % args[0], file=stderr)
    return 1

  template = dedent("""\
  sqlite::Value::String(
    to_translate.to_string() + "%s"
  ),
  """)
  elements = [
    [" [%]",  " {%}", " <%>"],
    ["  [%]", " {%}", " <%>"],
  ]

  with open(args[1], "w+") as f:
    f.write(
      "vec![" + "".join(chain.from_iterable([
        map(lambda x: template % "".join(x), permutations(e, i))
          for i in range(1, 4)
            for e in elements
      ])) + "]"
    )

  return 0

if __name__ == "__main__":
  exit(main(argv))
