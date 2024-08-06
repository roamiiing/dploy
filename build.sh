#!/bin/bash

# This script is intended to be used in development
# just to run commands

set -e

function exit_if_error {
	if [ $? -ne 0 ]; then
		echo "$1"
		exit 1
	fi
}

function help {
	echo "Usage: build.sh <subcommand>"
	echo ""
	echo "Subcommands:"
	echo "  install - compile and install dploy locally"
	echo "  help    - show this help message"
}

function install {
    cargo install --path . --profile dev
	exit_if_error "Failed to build website"
}

if [ "$#" -eq 0 ]; then
	echo "No subcommand provided"
	echo ""
	help
	exit 1
fi

case "$1" in
  install)
	install
	;;
  help)
	help
	;;
  *)
	echo "Invalid subcommand: $1"
	exit 1
	;;
esac
