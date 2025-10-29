# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 0.6.0

### Added
- gRPC client configuration options (connection timeouts, HTTP/2 keep-alive, window sizes)

### Changed
- Removed Vixen dependency, now using gRPC and parsing accounts directly
- **Store & Parser bumped to v0.8.0** - Updated to use Solana v3 with interface versions
- **Client & CLI remain at v0.7.0** - Kept on Solana v2.2.x with spl-token-2022 v9.0.0 for stability
- **Program remains at v0.7.0** - Kept on Solana v2.2.1 with pinocchio

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
