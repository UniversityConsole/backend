#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 -i <arg> -o <org>."
  exit 1
}

while getopts "i:o:" opt ; do
  case "$opt" in
    i ) INPUT_DIR=$(realpath "$OPTARG") ;;
    o ) OUTPUT_DIR="$OPTARG" ;;
    ? ) usage; exit 1 ;;
  esac
done

if [ -z "$INPUT_DIR" ] || [ -z "$OUTPUT_DIR" ]; then
  usage
fi

OUTPUT_DIR=$(realpath "$OUTPUT_DIR")
mkdir -p "$OUTPUT_DIR"

LAMBDA_BOOTSTRAPS=$(cargo metadata --no-deps --format-version 1 | jp "packages[?metadata != \`null\` && metadata.artifact_type == 'lambda_bootstrap'].name" | jq --raw-output '.[]')

printf "Found %d lambda bootstraps.\n" $(echo "$LAMBDA_BOOTSTRAPS" | wc -l)

echo "$LAMBDA_BOOTSTRAPS" | while read -r artifact ; do
  printf "Exporting artifact \"%s\".\n" "$artifact"
  mv "$INPUT_DIR/$artifact" "/tmp/bootstrap" \
    && zip -j "$OUTPUT_DIR/$artifact.zip" "/tmp/bootstrap"
done

echo "Output directory"
ls -lh "$OUTPUT_DIR"
