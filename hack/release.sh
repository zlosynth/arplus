#!/usr/bin/env bash
set -euo pipefail

version=${1}

sed -i "s/## Unreleased/## Unreleased\n\n## ${version}/" CHANGELOG.md
sed -i "s/version =.* # hack\/release.sh$/version = \"${version}\" # hack\/release.sh/" firmware/Cargo.toml
sed -i "s/rev .*/rev \"v${version}\")/" hardware/Module.kicad_sch
sed -i "s/gr_text \"board .*\" /gr_text \"board v${version}\" /" hardware/Module.kicad_pcb
sed -i "s/rev .*/rev \"v${version}\")/" hardware/Module.kicad_pcb

make

rm -rf release
mkdir release

pushd firmware && cargo objcopy --release -- -O binary ../release/arplus-firmware-${version}.bin && popd

make manual/user manual/build
cp manual/user/manual.pdf release/arplus-user-manual.pdf
cp manual/build/manual.pdf release/arplus-build-manual.pdf

export CHANGES=$(awk "/## ${version}/{flag=1;next}/## */{flag=0}flag" CHANGELOG.md | awk 'NF')
export BINARY=arplus-firmware-${version}.bin
envsubst < hack/release.tmpl.md > release/notes.md
