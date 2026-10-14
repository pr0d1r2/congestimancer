# ceiling-cap: no .context-limits ceiling may exceed its cap (nix:V3).
# A SPEC.md file row is capped at 3500 tokens, a directory (chain) row at
# 5500. A node that would pass its cap splits (`sherd split`); it does not
# raise.
!/^#/ && NF == 2 {
  cap = ($1 ~ /SPEC[.]md$/) ? 3500 : 5500
  if ($2 + 0 > cap) {
    print "ceiling-cap: " $1 " ceiling " $2 " is over the cap " cap
    bad = 1
  }
}
END { exit bad }
