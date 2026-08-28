// Package version exposes build-time version metadata.
//
// The values are set via -ldflags "-X github.com/ljh-sh/bpan/internal/version.Version=..."
// at link time. The release CI passes the git tag (without leading 'v'),
// the build timestamp (RFC3339), and the commit SHA.
package version

// These variables are overridden at build time via -ldflags.
var (
	// Version is the semantic version (e.g. "0.1.0"), set from the git tag.
	Version = "dev"
	// Commit is the git commit SHA at build time.
	Commit = "unknown"
	// BuildTime is the RFC3339-formatted build timestamp.
	BuildTime = "unknown"
	// GoVersion is the Go toolchain version that built the binary.
	GoVersion = "unknown"
)

// String returns a one-line human-readable version summary.
func String() string {
	return "bpan " + Version + " (commit " + Commit + ", built " + BuildTime + " with " + GoVersion + ")"
}