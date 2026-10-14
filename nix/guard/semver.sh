#!/usr/bin/env bash
#
# semver (nix:V24): the public API keeps its promises against the latest
# release tag. Before the first release there is no baseline; the step says
# so and passes, because the spec starts this check at the first publish.
#
#   nix/guard/semver.sh
set -euo pipefail

tag=$(git tag --list 'v*' --sort=-v:refname | head -n 1)
if [ -z "$tag" ]; then
  echo "semver: no v* release tag yet, so there is no baseline; this check starts with the first publish (nix:V24)."
  exit 0
fi
cargo semver-checks check-release -p congestimancer --baseline-rev "$tag"
