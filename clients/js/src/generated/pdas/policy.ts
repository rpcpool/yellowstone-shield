/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import {
  getAddressEncoder,
  getProgramDerivedAddress,
  getUtf8Encoder,
  type Address,
  type ProgramDerivedAddress,
} from '@solana/kit';

export type PolicySeeds = {
  /** The mint of the token extension account */
  mint: Address;
};

export async function findPolicyPda(
  seeds: PolicySeeds,
  config: { programAddress?: Address | undefined } = {}
): Promise<ProgramDerivedAddress> {
  const {
    programAddress = 'b1ockYL7X6sGtJzueDbxRVBEEPN4YeqoLW276R3MX8W' as Address<'b1ockYL7X6sGtJzueDbxRVBEEPN4YeqoLW276R3MX8W'>,
  } = config;
  return await getProgramDerivedAddress({
    programAddress,
    seeds: [
      getUtf8Encoder().encode('shield'),
      getUtf8Encoder().encode('policy'),
      getAddressEncoder().encode(seeds.mint),
    ],
  });
}
