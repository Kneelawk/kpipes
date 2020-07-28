set -ex

CROSS_VERSION=0.2.1

main() {
  # add custom docker images for linux
  if [ "$TARGET" = x86_64-unknown-linux-gnu ]; then
    case $CRATE_NAME in
    "kpipes-desktop")
      docker build -t kneelawk/cross-custom:x86_64-unknown-linux-gnu-$CROSS_VERSION ci/docker/x86_64-unknown-linux-gnu
      ;;
    "kpipes-qt")
      docker build -t kneelawk/cross-custom-qt:x86_64-unknown-linux-gnu-$CROSS_VERSION ci/docker/x86_64-unknown-linux-gnu-qt
      ;;
    esac
  fi
}

main
