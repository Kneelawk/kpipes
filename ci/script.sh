# This script takes care of testing the crate

set -ex

build_crate() {
  case $CRATE_NAME in
  "kpipes-desktop")
    ci/install_cross_config.sh base
    cross build --target "$TARGET" --package kpipes-desktop
    cross build --target "$TARGET" --package kpipes-desktop --release
    ci/install_cross_config.sh -u
    ;;
  "kpipes-qt")
    ci/install_cross_config.sh qt
    cross run --target "$TARGET" --package kpipes-qt-build -- --target "$TARGET" --profile debug
    cross run --target "$TARGET" --package kpipes-qt-build -- --target "$TARGET" --profile release
    ci/install_cross_config.sh -u
    ;;
  esac
}

test_crate() {
  case $CRATE_NAME in
  "kpipes-desktop")
    ci/install_cross_config.sh base
    cross test --target "$TARGET" --package kpipes-desktop
    cross test --target "$TARGET" --package kpipes-desktop --release
    ci/install_cross_config.sh -u
    ;;
  "kpipes-qt")
    ci/install_cross_config.sh qt
    cross test --target "$TARGET" --package kpipes-qt-rust
    cross test --target "$TARGET" --package kpipes-qt-rust --release
    ci/install_cross_config.sh -u
    ;;
  esac
}

main() {
  build_crate

  if [ -z "$DISABLE_TESTS" ]; then
    test_crate
  fi
}

# we don't run the "test phase" when doing deploys
if [ -z "$TRAVIS_TAG" ]; then
  main
fi
