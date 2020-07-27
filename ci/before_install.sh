set -ex

CROSS_VERSION=0.2.0

main() {
  # add custom docker images for linux
  if [ $TARGET = x86_64-unknown-linux-gnu ]; then
    docker build -t kneelawk/cross-custom:x86_64-unknown-linux-gnu-$CROSS_VERSION ci/docker/x86_64-unknown-linux-gnu
  fi
}

main
