# Handles installation of different cross configuration files.

set -ex

if [ "$1" = "-u" ]; then
  rm -f "Cross.toml"
else
  name="base"

  if [ -n "$1" ]; then
    name="$1"
  fi

  cp "ci/cross/Cross-$name.toml" "Cross.toml"
fi
