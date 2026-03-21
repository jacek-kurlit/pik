#!/bin/bash
set -e

VERSION=$1
FORMULA_PATH="Formula/pik.rb"

declare -A PLATFORMS=(
  ["macos_arm"]="aarch64-apple-darwin"
  ["macos_intel"]="x86_64-apple-darwin"
  ["linux_arm"]="aarch64-unknown-linux-gnu"
  ["linux_intel"]="x86_64-unknown-linux-gnu"
)

echo "Updating $FORMULA_PATH to version $VERSION..."

sed -i "s/version \".*\"/version \"$VERSION\"/" $FORMULA_PATH

for KEY in "${!PLATFORMS[@]}"; do
  ARCH_STR=${PLATFORMS[$KEY]}
  URL="https://github.com/jacek-kurlit/pik/releases/download/$VERSION/pik-$VERSION-$ARCH_STR.tar.gz"

  echo "Fetching SHA for $ARCH_STR..."
  NEW_SHA=$(curl -L -s "$URL.sha256" | awk '{print $1}')

  if [ -z "$NEW_SHA" ]; then
    echo "Error: Could not fetch $URL"
    exit 1
  fi

  sed -i "/$ARCH_STR.tar.gz/ {n; s/sha256 \".*\"/sha256 \"$NEW_SHA\"/;}" $FORMULA_PATH
done

echo "Formula updated successfully."
