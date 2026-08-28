// Package sandbox validates remote (Baidu Netdisk) paths to keep commands
// from escaping the user's expectation.
//
// Unlike bdpan-storage's /apps/bdpan/ filesystem sandbox, bpan operates
// against the whole Netdisk — users legitimately need to access any path.
// What we DO guard against is:
//   - empty or whitespace-only paths
//   - control characters in paths
//   - relative path traversal that would change the meaning unexpectedly
package sandbox

import (
	"fmt"
	"strings"
	"unicode"
)

// ErrInvalidPath is returned when a remote path fails validation.
type ErrInvalidPath struct {
	Path   string
	Reason string
}

func (e *ErrInvalidPath) Error() string {
	return fmt.Sprintf("sandbox: invalid remote path %q: %s", e.Path, e.Reason)
}

// NormalizeRemotePath returns a canonical absolute remote path.
//
// Rules:
//   - empty -> "/" (Netdisk root)
//   - must start with "/" (absolute)
//   - no NUL or control characters
//   - no backslash (Windows-style escapes are confusing on Linux)
//   - "." and ".." segments are resolved lexically
//
// This is purely lexical — it does not hit the network. The server still has
// the final say on whether the path exists.
func NormalizeRemotePath(p string) (string, error) {
	if strings.TrimSpace(p) == "" {
		return "/", nil
	}
	if !strings.HasPrefix(p, "/") {
		p = "/" + p
	}
	if strings.ContainsRune(p, '\\') {
		return "", &ErrInvalidPath{Path: p, Reason: "backslash not allowed"}
	}
	for _, r := range p {
		if r == 0 || (unicode.IsControl(r)) {
			return "", &ErrInvalidPath{Path: p, Reason: "control character"}
		}
	}

	// Lexical normalization of "." and ".." segments.
	segs := strings.Split(p, "/")
	out := make([]string, 0, len(segs))
	for _, s := range segs {
		switch s {
		case "", ".":
			continue
		case "..":
			if len(out) == 0 {
				return "", &ErrInvalidPath{Path: p, Reason: "traverses above Netdisk root"}
			}
			out = out[:len(out)-1]
		default:
			out = append(out, s)
		}
	}
	if len(out) == 0 {
		return "/", nil
	}
	return "/" + strings.Join(out, "/"), nil
}

// JoinRemote joins a parent directory and a leaf, normalizing the result.
func JoinRemote(parent, leaf string) (string, error) {
	parent, err := NormalizeRemotePath(parent)
	if err != nil {
		return "", err
	}
	leaf = strings.TrimLeft(leaf, "/")
	candidate := parent + "/" + leaf
	return NormalizeRemotePath(candidate)
}