#!/usr/bin/env bash
if [[ "$OS" == "Windows_NT" ]]; then
  ./pm.bat "$@"
else
  ./pm.sh "$@"
fi