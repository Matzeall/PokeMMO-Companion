#!/usr/bin/env bash
set -euo pipefail

../target/debug/data-builder data/source/EN/ data/source/EN/ --out data/locales/EN/ --base-data-out data/base/
../target/debug/data-builder data/source/EN/ data/source/DE/ --out data/locales/DE/
../target/debug/data-builder data/source/EN/ data/source/IT/ --out data/locales/IT/
../target/debug/data-builder data/source/EN/ data/source/FR/ --out data/locales/FR/
../target/debug/data-builder data/source/EN/ data/source/ES/ --out data/locales/ES/
