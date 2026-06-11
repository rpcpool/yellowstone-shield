# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 0.7.0

### Changed

- Bump Solana dependencies to Agave v4.0 (#43)
- Bump yellowstone-grpc-client to v13.1 and yellowstone-grpc-proto to v12.4 (#43)
- Bump the Solana CLI used for program builds and the local validator to v4.0.2 (#43)
- Update rust-toolchain to 1.93.1 and declare `rust-version` on the published crates (#43)
- Reorganize workspace dependencies into Agave, Solana SDK, SPL, and Yellowstone sections, and remove unused ones (#43)
- Adapt the policy store to the v4 RPC client API (`get_program_ui_accounts_with_config`) (#43)
- The parser now depends on solana-pubkey directly instead of solana-program (#43)

## 0.6.0

### Changed

- Bump Solana dependencies to v3.0 (#38)
- Update anchor-lang to v0.32.1
- Update client generators and regenerate clients
- Update rust-toolchain to 1.86

## 0.5.1

### Changed

- Restructure crates dependencies to use workspace dependencies instead.

### Added

- New `update` command to batch update or replace the policy identities list.

### Changed

- Enhanced `add` command to **automatically replace removed identities** instead of appending, when available
- Include displaying identity details while logging policy details after command completion

## 0.5.0 - 06/19/2025

### Added

- Added PolicyV2 which keeps the TE mint on the policy so can find metadata of the policy from its account. (https://github.com/rpcpool/yellowstone-shield/pull/12)

### Changed

- Update program, SDK, CLI, and parser to be backward compatible with `Policy` and `PolicyV2` accounts.
- Refactoring to the program to consolidate validations and cover out of bound account errors. (https://github.com/rpcpool/yellowstone-shield/pull/13)
- Revised the public API of the policy store to include a dedicated configuration structure. (https://github.com/rpcpool/yellowstone-shield/pull/9)

### Fixed

- `identities_len` is now the count of active identities on policy v2 and not the total length of the identities slice.
