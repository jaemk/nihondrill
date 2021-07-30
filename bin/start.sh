#!/bin/bash

set -e

BUILD=false
RELEASE=false

function die_usage {
  echo "Usage: $(basename $0) [-bh]"
  echo ""
  echo "Options:"
  echo "  -h           Show this beautiful message"
  echo "  -b           Build the artifacts if they're missing"
  echo "  -r           Build in release mode"
  echo ""
  exit 1
}

while getopts "bhr" opt; do
  case $opt in
    b) BUILD=true ;;
    r) RELEASE=true ;;
    h) die_usage ;;
    \? ) die_usage
      ;;
  esac
done

if $BUILD || $RELEASE; then
  if $RELEASE; then
    cargo build --release
    cp target/release/nihondrill bin/nihondrill
    echo "built (release)"
  else
    cargo build
    cp target/debug/nihondrill bin/nihondrill
    echo "built (debug)"
  fi
fi

if [[ ! -f bin/nihondrill ]]; then
  echo "missing bin/nihondrill executable"
  exit 1
fi

migrant setup
migrant list
migrant apply -a || true

./bin/nihondrill

