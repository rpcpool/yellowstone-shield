#!/usr/bin/env zx
import 'zx/globals';
import { getCargo, getProgramFolders } from './utils.mjs';


for (const folder of getProgramFolders()) {
  const cargo = getCargo(folder);
  const programDir = path.join(__dirname, '..', folder);
  const programId = cargo.package.metadata.solana['program-id'];

  await $`shank idl --program-id="${programId}" --crate-root="${programDir}" --out-filename="idl.json" --out-dir="${programDir}"`
}