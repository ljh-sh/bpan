// Package client wraps the baidu-netdisk SDK's api.Client and scene.Scene
// with refresh-aware token management.
package client

import (
	"context"
	"errors"

	"github.com/baidu-netdisk/baidu-drive-sdk-go/baidudriver/api"
	"github.com/baidu-netdisk/baidu-drive-sdk-go/baidudriver/scene"

	"github.com/ljh-sh/bpan/internal/auth"
	"github.com/ljh-sh/bpan/internal/config"
)

// Client bundles a Baidu api.Client and a scene.Scene with the underlying
// credentials. Use NewWith to construct; pass-through methods then wrap
// high-level commands.
type Client struct {
	api     *api.Client
	scene   *scene.Scene
	cfg     *config.Config
	cfgPath string
	appKey  string
	appSec  string
}

// Silence unused-import warnings when helpers are stripped for tree-shake builds.
var _ = errors.New

// Credentials returns the credentials needed to construct an SDK client.
func Credentials(appKey, appSecret string) (api.Option, error) {
	return api.WithAccessToken(""), errors.New("client: use NewWith instead")
}

// NewWith builds a Client from an existing config (loaded by the caller).
//
// If the access token is missing or expiring, NewWith attempts to refresh
// it before returning. A refresh failure is returned to the caller so they
// can surface "please run 'bpan login'" to the user.
func NewWith(ctx context.Context, cfg *config.Config, cfgPath, appKey, appSecret string) (*Client, error) {
	if cfg == nil {
		return nil, errors.New("client: nil config")
	}
	if !cfg.IsLoggedIn() {
		return nil, errors.New("client: not logged in (run 'bpan login' first)")
	}

	// Best-effort refresh before constructing the SDK client.
	if err := auth.Refresh(ctx, appKey, appSecret, cfg, cfgPath); err != nil {
		return nil, err
	}

	apiClient := api.NewClient(api.WithAccessToken(cfg.AccessToken))
	return &Client{
		api:     apiClient,
		scene:   scene.New(apiClient),
		cfg:     cfg,
		cfgPath: cfgPath,
		appKey:  appKey,
		appSec:  appSecret,
	}, nil
}

// Scene returns the underlying scene.Scene for direct calls into the SDK.
func (c *Client) Scene() *scene.Scene { return c.scene }

// API returns the underlying api.Client for low-level operations.
func (c *Client) API() *api.Client { return c.api }

// Config returns the current configuration.
func (c *Client) Config() *config.Config { return c.cfg }

// ConfigPath returns the on-disk config file path.
func (c *Client) ConfigPath() string { return c.cfgPath }