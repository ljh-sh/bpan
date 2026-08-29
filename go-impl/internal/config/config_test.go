package config

import (
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestLoadMissing(t *testing.T) {
	dir := t.TempDir()
	cfg, err := Load(filepath.Join(dir, "config.json"))
	if err != nil {
		t.Fatal(err)
	}
	if cfg.Version != CurrentVersion {
		t.Errorf("default version=%d want %d", cfg.Version, CurrentVersion)
	}
	if cfg.IsLoggedIn() {
		t.Error("fresh config must not be logged in")
	}
}

func TestSaveLoadRoundTrip(t *testing.T) {
	dir := t.TempDir()
	p := filepath.Join(dir, "config.json")
	in := &Config{
		Version:      CurrentVersion,
		ClientID:     "abc",
		AccessToken:  "tok",
		RefreshToken: "ref",
		ExpiresAt:    time.Now().Add(time.Hour).Truncate(time.Second),
		Scope:        "basic,netdisk",
		User:         &UserInfo{BaiduName: "alice", UK: 1, VipLevel: 2},
	}
	if err := Save(p, in); err != nil {
		t.Fatal(err)
	}
	// File must be 0600.
	st, err := os.Stat(p)
	if err != nil {
		t.Fatal(err)
	}
	if perm := st.Mode().Perm(); perm != 0o600 {
		t.Errorf("config perm=%o want 0600", perm)
	}

	out, err := Load(p)
	if err != nil {
		t.Fatal(err)
	}
	if out.AccessToken != "tok" || out.RefreshToken != "ref" || out.User.BaiduName != "alice" {
		t.Errorf("round-trip mismatch: %+v", out)
	}
}

func TestIsAccessTokenValid(t *testing.T) {
	c := &Config{AccessToken: "x", ExpiresAt: time.Now().Add(time.Hour)}
	if !c.IsAccessTokenValid() {
		t.Error("token in 1h must be valid")
	}
	c.ExpiresAt = time.Now().Add(time.Minute) // within safety margin
	if c.IsAccessTokenValid() {
		t.Error("token in 1m must NOT be valid (within 5min safety margin)")
	}
	c.ExpiresAt = time.Time{}
	if !c.IsAccessTokenValid() {
		t.Error("token with no expiry must be assumed valid")
	}
	c.AccessToken = ""
	if c.IsAccessTokenValid() {
		t.Error("missing token must be invalid")
	}
}

func TestNeedsRefresh(t *testing.T) {
	c := &Config{AccessToken: "x", RefreshToken: "r", ExpiresAt: time.Now().Add(time.Hour)}
	if c.NeedsRefresh() {
		t.Error("fresh token must not need refresh")
	}
	c.ExpiresAt = time.Now()
	if !c.NeedsRefresh() {
		t.Error("expired token must need refresh")
	}
	c.RefreshToken = ""
	if c.NeedsRefresh() {
		t.Error("no refresh_token means nothing to refresh")
	}
}