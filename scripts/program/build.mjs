#!/usr/bin/env zx
import 'zx/globals';
import {
  cliArguments,
  getProgramFolders,
  workingDirectory,
} from '../utils.mjs';

// Save external programs binaries to the output directory.
import './dump.mjs';

// Configure additional arguments here, e.g.:
// ['--arg1', '--arg2', ...cliArguments()]
const buildArgs = cliArguments();

// Build the programs. The output directory is forced to the workspace-level
// target/deploy folder because the program package lives outside the
// workspace and would otherwise emit into its own target directory.
const sbfOutDir = path.join(workingDirectory, 'target', 'deploy');
for (const folder of getProgramFolders()) {
  const manifestPath = path.join(workingDirectory, folder, 'Cargo.toml');

  await $`cargo-build-sbf --manifest-path ${manifestPath} --sbf-out-dir ${sbfOutDir} ${buildArgs}`;
}
