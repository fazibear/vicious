#!/bin/bash

VER=82
EXT=7z
URL="https://hvsc.brona.dk/HVSC/HVSC_${VER}-all-of-them.${EXT}"
TMP_FILE="hvsc.${EXT}"
MUSIC_DIR="C64Music"

echo "Downloading HVSC collection from ${URL}..."
curl ${URL} > ${TMP_FILE}
echo "Unzipping ..."
7z x -y ${TMP_FILE}
rm -f ${TMP_FILE}
rm -rf ./$MUSIC_DIR/update
rm -rf ./$MUSIC_DIR/DOCUMENTS
find ./$MUSIC_DIR ! -name '*.sid' -type f -exec rm -f {} +
