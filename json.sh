#!/bin/bash

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

dir_to_json C64Music > C64Music.json
