#!/bin/bash
set -e

VERSION=$1
MANIFEST_PATH="bucket/pik.json"

echo "Updating $MANIFEST_PATH to version $VERSION..."

URL="https://github.com/jacek-kurlit/pik/releases/download/$VERSION/pik-$VERSION-x86_64-pc-windows-msvc.zip"
SHA=$(curl -L -s "$URL.sha256" | awk '{print $1}')

if [ -z "$SHA" ]; then
  echo "Error: Could not fetch SHA256 for $URL"
  exit 1
fi

sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" $MANIFEST_PATH
sed -i "s|\"url\": \"https://github.com/jacek-kurlit/pik/releases/download/.*\.zip\"|\"url\": \"$URL\"|" $MANIFEST_PATH
sed -i "s/\"hash\": \".*\"/\"hash\": \"$SHA\"/" $MANIFEST_PATH

echo "Manifest updated successfully."
