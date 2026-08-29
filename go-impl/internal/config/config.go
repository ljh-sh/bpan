// Package config manages the bpan configuration file.
//
// The configuration file lives at ~/.config/bdpan/config.json (0600) and is
// compatible with baidu-netdisk/bdpan-storage for interoperability.
package config

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"time"
)

// Config is the on-disk representation of bpan's configuration.
type Config struct {
	Version      int       `json:"version"`
	ClientID     string    `json:"client_id"`
	AccessToken  string    `json:"access_token"`
	RefreshToken string    `json:"refresh_token"`
	ExpiresAt    time.Time `json:"expires_at"`
	Scope        string    `json:"scope"`
	User         *UserInfo `json:"user,omitempty"`
}

// UserInfo caches the latest known user profile (best-effort, refreshed on each login).
type UserInfo struct {
	BaiduName   string `json:"baidu_name"`
	UK          int64  `json:"uk"`
	VipLevel    int    `json:"vip_level"`
	AvatarURL   string `json:"avatar_url,omitempty"`
	NetdiskName string `json:"netdisk_name,omitempty"`
}

// CurrentVersion is the config schema version this binary writes.
const CurrentVersion = 1

// DefaultPath returns the default config file path under the user's home directory.
func DefaultPath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("config: cannot determine home directory: %w", err)
	}
	return filepath.Join(home, ".config", "bdpan", "config.json"), nil
}

// Load reads and parses the config file at the given path.
//
// If the file does not exist, returns a fresh zero-valued Config (not an error).
// This allows first-run flows to start with an empty config and write it later.
func Load(path string) (*Config, error) {
	f, err := os.Open(path) // #nosec G304 -- path comes from caller-controlled flag or default
	if err != nil {
		if os.IsNotExist(err) {
			return &Config{Version: CurrentVersion}, nil
		}
		return nil, fmt.Errorf("config: open %s: %w", path, err)
	}
	defer f.Close()

	data, err := io.ReadAll(f)
	if err != nil {
		return nil, fmt.Errorf("config: read %s: %w", path, err)
	}

	if len(data) == 0 {
		return &Config{Version: CurrentVersion}, nil
	}

	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("config: parse %s: %w", path, err)
	}
	return &cfg, nil
}

// Save atomically writes the config to path with 0600 permissions.
//
// The write goes via a sibling temp file followed by rename, so a crash mid-write
// does not leave a half-written config that would prevent the next login.
func Save(path string, cfg *Config) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o700); err != nil {
		return fmt.Errorf("config: mkdir %s: %w", filepath.Dir(path), err)
	}

	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return fmt.Errorf("config: marshal: %w", err)
	}

	tmp := path + ".tmp"
	if err := os.WriteFile(tmp, data, 0o600); err != nil {
		return fmt.Errorf("config: write %s: %w", tmp, err)
	}
	if err := os.Rename(tmp, path); err != nil {
		_ = os.Remove(tmp)
		return fmt.Errorf("config: rename %s: %w", path, err)
	}
	return nil
}

// IsLoggedIn returns true if the config has an access token.
//
// Note: this does NOT check token expiry — a token may be present but expired.
// Callers that need a guarantee of validity should use IsAccessTokenValid.
func (c *Config) IsLoggedIn() bool {
	return c != nil && c.AccessToken != ""
}

// IsAccessTokenValid returns true if the access token exists and is not within
// the safety margin of expiry. The safety margin (default 5 minutes) prevents
// issuing requests that will fail with a 401 mid-call.
func (c *Config) IsAccessTokenValid() bool {
	if !c.IsLoggedIn() {
		return false
	}
	if c.ExpiresAt.IsZero() {
		// No expiry info — assume still valid; the server will tell us otherwise.
		return true
	}
	return time.Now().Add(5 * time.Minute).Before(c.ExpiresAt)
}

// NeedsRefresh returns true if the access token should be refreshed now.
func (c *Config) NeedsRefresh() bool {
	if !c.IsLoggedIn() {
		return false
	}
	if c.ExpiresAt.IsZero() || c.RefreshToken == "" {
		return false
	}
	return time.Now().Add(5 * time.Minute).After(c.ExpiresAt)
}

// Delete removes the config file (logout).
func Delete(path string) error {
	err := os.Remove(path)
	if err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("config: remove %s: %w", path, err)
	}
	return nil
}