#!/usr/bin/env bash
set -euo pipefail

adb devices -l

(
  cd examples/playground
  nsh test e2e.test.tsx --target android 2>&1 | tee "$RUNNER_TEMP/android-playground.log"
  grep -q '\[e2e\] platform=android ' "$RUNNER_TEMP/android-playground.log"
)

(
  cd examples/denied-cap
  nsh test e2e.test.tsx --target android 2>&1 | tee "$RUNNER_TEMP/android-denied-cap.log"
  grep -q '\[e2e\] platform=android ' "$RUNNER_TEMP/android-denied-cap.log"
)
