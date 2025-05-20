#!/bin/bash

VER=82
EXT=7z
URL="https://hvsc.brona.dk/HVSC/HVSC_${VER}-all-of-them.${EXT}"
TMP_FILE="hvsc.${EXT}"
MUSIC_DIR="C64Music"

download_and_unzip() {
  if [ -d $MUSIC_DIR ]; then
    return
  fi

  echo "Downloading HVSC collection from ${URL}..."
  curl ${URL} > ${TMP_FILE}
  echo "Unzipping ..."
  7z x -y ${TMP_FILE}
  rm -f ${TMP_FILE}
  rm -rf ./$MUSIC_DIR/update
  rm -rf ./$MUSIC_DIR/DOCUMENTS
  find ./$MUSIC_DIR ! -name '*.sid' -type f -exec rm -f {} +
}

json_escape() {
  sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'
}

dir_to_json() {
  local path="$1"
  local indent="${2:-  }"

  if [ -d "$path" ]; then
    echo "{"
    echo "${indent}\"type\": \"directory\","
    echo "${indent}\"name\": \"$(basename "$path" | json_escape)\","
    echo "${indent}\"path\": \"$(echo "$path" | json_escape)\","
    echo "${indent}\"children\": ["
    local first=1
    for entry in "$path"/*; do
      [ -e "$entry" ] || continue
      if [ $first -eq 0 ]; then
        echo ","
      fi
      first=0
      echo -n "${indent}  "
      dir_to_json "$entry" "  $indent"
    done
    echo
    echo "${indent}]"
    echo -n "}"
  else
    [ -f /*.sid ] && continue
    echo -n "{"
    echo -n "\"type\": \"file\", "
    echo -n "\"name\": \"$(basename "$path" | json_escape)\", "
    echo -n "\"path\": \"$(echo "$path" | json_escape)\""
    echo -n "}"
  fi
}

download_and_unzip
#dir_to_json $MUSIC_DIR > ${MUSIC_DIR}.json
./json.rb > ${MUSIC_DIR}.json
