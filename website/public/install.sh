#!/bin/sh

# This is install script for dploy
# https://github.com/roamiiing/dploy

# Define current arch
ARCH=$(uname -m)
OSTYPE=$(uname -s)

case "$ARCH" in
  x86_64)
	GREP_ARCH_PATTERN=x86_64
	;;
  arm64)
	GREP_ARCH_PATTERN=aarch64
	;;
  *)
	echo "Unsupported architecture: $ARCH"
	exit 1
	;;
esac

case "$OSTYPE" in
  Darwin*)
	GREP_OS_PATTERN=macos
	;;
  Linux*)
	GREP_OS_PATTERN=linux
	;;
  *)
	echo "Unsupported OS: $OSTYPE"
	exit 1
	;;
esac

BIN_FOLDER_NAME=bins-$GREP_ARCH_PATTERN-$GREP_OS_PATTERN

GREP_PATTERN="$BIN_FOLDER_NAME.tar.gz"

DOWNLOAD_URL=$(
	curl -s https://api.github.com/repos/roamiiing/dploy/releases/latest \
	| grep -o "https.*$GREP_PATTERN" \
)

echo "Downloading binary from $DOWNLOAD_URL"

curl -L -o /tmp/dploy.tar.gz $DOWNLOAD_URL

echo "Extracting binary"

mkdir -p /tmp/dploy
tar -xzf /tmp/dploy.tar.gz -C /tmp/dploy
mkdir -p $HOME/.dploy/bin
mv /tmp/dploy/dist/$BIN_FOLDER_NAME/dploy $HOME/.dploy/bin 
chmod +x $HOME/.dploy/bin/dploy

rm -rf /tmp/dploy

echo ""
echo "Done installing dploy"
echo "Note that you need to add $HOME/.dploy/bin to your PATH"
echo ""

echo "For example:"
echo "  echo 'export PATH=\$HOME/.dploy/bin:\$PATH' >> \$HOME/.zshrc"
echo "  echo 'export PATH=\$HOME/.dploy/bin:\$PATH' >> \$HOME/.bashrc"
echo "Or any of your favorite shell (such as fish)"

