// Package auth handles the OAuth device-code flow against the Baidu Netdisk
// Open Platform, and token refresh.
//
// We use the device-code grant (RFC 8628) rather than the authorization-code
// grant because:
//   - it requires no loopback HTTP server (no port conflicts, no pre-registered
//     redirect_uri, works in SSH/containers);
//   - it survives intermittent connectivity on the CLI side (we poll until
//     the user authorizes); and
//   - the SDK already provides DeviceCode / DeviceToken helpers, so the
//     implementation is short.
package auth

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/baidu-netdisk/baidu-drive-sdk-go/baidudriver/api"

	"github.com/ljh-sh/bpan/internal/config"
)

// DeviceCode is the prompt shown to the user during login.
type DeviceCode struct {
	UserCode        string
	VerificationURL string
	ExpiresAt       time.Time
}

// ErrDeviceCodeExpired is returned when the user takes too long to authorize.
var ErrDeviceCodeExpired = errors.New("auth: device code expired before user authorized")

// ErrAuthorizationPending is returned when polling and the user has not yet
// authorized. Callers should sleep for the device code's Interval and retry.
var ErrAuthorizationPending = errors.New("auth: authorization pending")

// Login walks the user through the device-code flow and writes the resulting
// tokens into cfg. It blocks until the user authorizes, the device code expires,
// or ctx is cancelled.
//
//	appKey    — Baidu Open Platform AppKey (a.k.a. client_id)
//	secretKey — Baidu Open Platform SecretKey (a.k.a. client_secret)
//	cfg       — config to mutate and save
//	cfgPath   — where to save the updated config
func Login(ctx context.Context, appKey, secretKey string, cfg *config.Config, cfgPath string) error {
	if appKey == "" {
		return errors.New("auth: appKey (client_id) is required (set BDPAN_CLIENT_ID or use --app-key)")
	}
	if secretKey == "" {
		return errors.New("auth: secretKey (client_secret) is required (set BDPAN_CLIENT_SECRET or use --app-key)")
	}

	c := api.NewClient()

	// Step 1: request a device code.
	dc, err := c.Auth.DeviceCode(ctx, appKey)
	if err != nil {
		return fmt.Errorf("auth: DeviceCode: %w", err)
	}

	prompt := DeviceCode{
		UserCode:        dc.UserCode,
		VerificationURL: dc.VerificationURL,
		ExpiresAt:       time.Now().Add(time.Duration(dc.ExpiresIn) * time.Second),
	}
	interval := time.Duration(dc.Interval) * time.Second
	if interval < time.Second {
		interval = 5 * time.Second
	}

	fmt.Printf("\nTo authorize bpan, open this URL in your browser:\n\n")
	fmt.Printf("    %s\n\n", prompt.VerificationURL)
	fmt.Printf("And enter this code:\n\n")
	fmt.Printf("    %s\n\n", prompt.UserCode)
	fmt.Printf("Waiting for authorization (expires at %s)...\n", prompt.ExpiresAt.Format(time.RFC3339))

	// Step 2: poll until the user authorizes, the code expires, or ctx is cancelled.
	deadline := time.NewTimer(time.Until(prompt.ExpiresAt))
	defer deadline.Stop()

	poll := time.NewTicker(interval)
	defer poll.Stop()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-deadline.C:
			return ErrDeviceCodeExpired
		case <-poll.C:
			tok, err := c.Auth.DeviceToken(ctx, appKey, secretKey, dc.DeviceCode)
			if err != nil {
				var apiErr *api.APIError
				if errors.As(err, &apiErr) {
					switch apiErr.Errno {
					case 110:
						// 110: authorization pending — user hasn't approved yet, keep polling
						continue
					case 111:
						// expired_token: device code expired; restart from step 1
						return ErrDeviceCodeExpired
					}
				}
				return fmt.Errorf("auth: DeviceToken poll: %w", err)
			}

			cfg.Version = config.CurrentVersion
			cfg.ClientID = appKey
			cfg.AccessToken = tok.AccessToken
			cfg.RefreshToken = tok.RefreshToken
			cfg.Scope = tok.Scope
			if tok.ExpiresIn > 0 {
				cfg.ExpiresAt = time.Now().Add(time.Duration(tok.ExpiresIn) * time.Second)
			}

			if err := config.Save(cfgPath, cfg); err != nil {
				return fmt.Errorf("auth: save config: %w", err)
			}

			fmt.Println("✓ Authorization successful — credentials saved.")
			return nil
		}
	}
}

// Refresh exchanges the refresh_token for a fresh access_token.
//
// Returns nil if the refresh succeeds and the cfg has been updated and saved.
// Returns the original (unmodified) cfg if the token does not need refresh.
//
// If the refresh_token itself has been revoked, the cfg is wiped and an error
// is returned so the caller can prompt the user to log in again.
func Refresh(ctx context.Context, appKey, secretKey string, cfg *config.Config, cfgPath string) error {
	if !cfg.NeedsRefresh() {
		return nil
	}
	if appKey == "" || secretKey == "" {
		return errors.New("auth: refresh requires client_id and client_secret in env or --app-key")
	}

	c := api.NewClient()
	tok, err := c.Auth.Code2Token(ctx, appKey, secretKey, cfg.RefreshToken, "oob")
	if err != nil {
		var apiErr *api.APIError
		if errors.As(err, &apiErr) && (apiErr.Errno == 111 || apiErr.Errno == 110) {
			// refresh_token invalid — wipe and force re-login
			_ = config.Delete(cfgPath)
			cfg.AccessToken = ""
			cfg.RefreshToken = ""
			cfg.ExpiresAt = time.Time{}
			return errors.New("auth: refresh_token invalid; please run 'bpan login' again")
		}
		return fmt.Errorf("auth: refresh: %w", err)
	}

	cfg.AccessToken = tok.AccessToken
	if tok.RefreshToken != "" {
		cfg.RefreshToken = tok.RefreshToken
	}
	if tok.ExpiresIn > 0 {
		cfg.ExpiresAt = time.Now().Add(time.Duration(tok.ExpiresIn) * time.Second)
	}

	if err := config.Save(cfgPath, cfg); err != nil {
		return fmt.Errorf("auth: save refreshed config: %w", err)
	}
	return nil
}