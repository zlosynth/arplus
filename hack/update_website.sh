#!/usr/bin/env bash
set -euo pipefail

make manual/user
make manual/build

cp manual/user/manual.pdf ../zlosynth.github.io/docs/arplus-user-manual.pdf
cp manual/build/manual.pdf ../zlosynth.github.io/docs/arplus-build-manual.pdf
