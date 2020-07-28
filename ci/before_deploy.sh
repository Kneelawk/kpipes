# This script takes care of building the crate and packaging it for release

set -ex

build_crate() {
  case $CRATE_NAME in
  "kpipes-desktop")
    ci/install_cross_config.sh base
    cross build --target "$TARGET" --package kpipes-desktop --release
    ci/install_cross_config.sh -u
    ;;
  "kpipes-qt")
    ci/install_cross_config.sh qt
    cross run --target "$TARGET" --package kpipes-qt-build -- --target "$TARGET" --profile release
    ci/install_cross_config.sh -u
    ;;
  esac
}

install_crate() {
  local stage="$1"

  case $CRATE_NAME in
  "kpipes-desktop")
    cp target/"$TARGET"/release/kpipes-desktop "$stage"/
    ;;
  "kpipes-qt")
    for file in target/"$TARGET"/release/cmake-build/*
    do
      if [[ "$file" != *build ]]; then
        cp -r "$file" "$stage"/
      fi
    done
    ;;
  esac
}

main() {
  local src=$(pwd) \
  stage=

  case $TRAVIS_OS_NAME in
  linux)
    stage="$(mktemp -d)"
    ;;
  osx)
    stage="$(mktemp -d -t tmp)"
    ;;
  esac

  test -f Cargo.lock || cargo generate-lockfile

  build_crate

  install_crate "$stage"

  cd "$stage"
  tar czf "$src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz" ./*
  cd "$src"

  rm -rf "$stage"
}

main
