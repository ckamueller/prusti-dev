#!/bin/bash

# Usage: script <crate/download/dir> <file/with/list/of/crates> <verification_log_date>

set -eo pipefail

info() { echo -e "[-] ($(date '+%Y-%m-%d %H:%M:%S')) ${*}"; }
error() { echo -e "[!] ($(date '+%Y-%m-%d %H:%M:%S')) ${*}"; }

cargoclean() {
	# Clean the artifacts of this project ("bin" or "lib"), but not those of the dependencies
	names="$(cargo metadata --format-version 1 | jq -r '.packages[].targets[] | select( .kind | map(. == "bin" or . == "lib") | any ) | select ( .src_path | contains(".cargo/registry") | . != true ) | .name')"
	for name in $names; do
		cargo clean -p "$name" || cargo clean
	done
}

info "=== Coarse-grained verification ==="

# Get the directory in which this script is contained
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"

# Get the folder in which all the crates has been downloaded
CRATE_DOWNLOAD_DIR="$(realpath "$1")"
if [[ ! -d "$CRATE_DOWNLOAD_DIR/000_libc" ]]; then
	echo "It looks like CRATE_DOWNLOAD_DIR (first argument) is wrong: '$CRATE_DOWNLOAD_DIR'"
	exit 1
fi

# Get the file with the list of crates to compile
CRATES_LIST_PATH="$(realpath "$2")"
if [[ ! -r "$CRATES_LIST_PATH" ]]; then
	error "Could not read file '$CRATES_LIST_PATH' (second argument)"
	exit 1
fi

# Get the filename of the verification log
VERIFICATION_LOG_DATE="$3"

verification_report="$CRATE_DOWNLOAD_DIR/verification-time-report-$VERIFICATION_LOG_DATE.csv"
verification_report_final="$CRATE_DOWNLOAD_DIR/verification-time-report.csv"
echo 'Crate name,Number of dependencies,Number of verified dependencies,Parsing duration,Type-checking duration,Encoding duration,Succesfully verified items,Verification duration' > "$verification_report"
info "Report: '$verification_report'"

info "Run on $(cat "$CRATES_LIST_PATH" | wc -l) crates"

cat "$CRATES_LIST_PATH" | while read crate_name; do
	info "=== Crate '$crate_name' ==="
	CRATE_DIR="$CRATE_DOWNLOAD_DIR/$crate_name"
	CRATE_ROOT="$CRATE_DIR/source"
	cd "$CRATE_ROOT"

	VERIFICATION_LOG_FILE="$CRATE_DIR/verify-coarse-grained-$VERIFICATION_LOG_DATE.log"

	if [[ ! -r "$VERIFICATION_LOG_FILE" ]]
	then
		error "Can not read VERIFICATION_LOG_FILE='$VERIFICATION_LOG_FILE'"
		continue
	fi

	verified_items="$( (egrep 'Received [0-9]+ items to be verified' "$VERIFICATION_LOG_FILE" | cut -d ' ' -f 6 | sed 's/$/+/' | tr '\n' ' ' ; echo "0") | bc )"
	successful_items="$( (egrep 'Successful verification of [0-9]+ items' "$VERIFICATION_LOG_FILE" | cut -d ' ' -f 4 | sed 's/$/+/' | tr '\n' ' ' ; echo "0") | bc )"

	parsing_duration="$( (egrep 'Parsing of annotations successful \(.* seconds\)' "$VERIFICATION_LOG_FILE" | cut -d ' ' -f 9 | sed 's/(//' | tr '\n' '+' ; echo 0) | bc )"
	type_checking_duration="$( (egrep 'Type-checking of annotations successful \(.* seconds\)' "$VERIFICATION_LOG_FILE" | cut -d ' ' -f 9 | sed 's/(//' | tr '\n' '+' ; echo 0) | bc )"
	encoding_duration="$( (egrep 'Encoding to Viper successful \(.* seconds\)' "$VERIFICATION_LOG_FILE" | cut -d ' ' -f 9 | sed 's/(//' | tr '\n' '+' ; echo 0) | bc )"
	verification_duration="$( (egrep 'Verification complete \(.* seconds\)' "$VERIFICATION_LOG_FILE" | cut -d ' ' -f 7 | sed 's/(//' | tr '\n' '+' ; echo 0) | bc )"
	num_dependencies="$( egrep 'Successful verification of [0-9]* items' "$VERIFICATION_LOG_FILE" | wc -l || true )"
	num_verified_dependencies="$( egrep 'Verification complete \(.* seconds\)' "$VERIFICATION_LOG_FILE" | wc -l || true )"

	info "Verification runs: $num_dependencies, verified items $verified_items in $verification_duration s"
	echo "$crate_name,$num_dependencies,$num_verified_dependencies,$parsing_duration,$type_checking_duration,$encoding_duration,$successful_items,$verification_duration" >> "$verification_report"
done

if [[ -r "$verification_report" ]]
then
	cp "$verification_report" "$verification_report_final"
fi
