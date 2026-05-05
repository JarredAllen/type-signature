# Changelog

## [Unreleased]

### Fixed
- Fixed unconditional impl blocks for atomic types which may or may not exist on some architectures.

## [0.1.2] - 2026-05-04

### Added
- Support for accessing the `type_signature` crate via a renamed/re-exported path from the derive
  macro.

### Fixed
- The `impl_type_signature_as!` macro now works correctly when re-exported and called from a crate
  which doesn't depend on us.

## [0.1.1] - 2026-05-04

### Added
- `impl_type_signature_as!` macro for implementing with custom fields/variants.

## [0.1.1]

## [0.1.0]

- Initial release.

[Unreleased]: https://github.com/JarredAllen/type-signature/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/JarredAllen/type-signature/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/JarredAllen/type-signature/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/JarredAllen/type-signature/releases/tag/v0.1.0
