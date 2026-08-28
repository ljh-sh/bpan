// Command bpan is a CLI for the Baidu Netdisk personal Open Platform.
//
// bpan vendors the baidu-netdisk/baidu-drive-sdk-go via git subtree, so it
// remains buildable even if Baidu takes the upstream repository private.
//
// The CLI exposes a small set of high-level commands (login, ls, upload,
// download, mkdir, rm, mv, cp, rename, quota) built on the SDK's scene layer.
// More commands (transfer, share, search) will land in v0.2.0 as the SDK's
// surface stabilizes.
package main

import (
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"os/signal"
	"path/filepath"
	"strconv"
	"strings"
	"syscall"
	"text/tabwriter"
	"time"

	"github.com/baidu-netdisk/baidu-drive-sdk-go/baidudriver/api"
	"github.com/baidu-netdisk/baidu-drive-sdk-go/baidudriver/scene"

	"github.com/ljh-sh/bpan/internal/auth"
	"github.com/ljh-sh/bpan/internal/client"
	"github.com/ljh-sh/bpan/internal/config"
	"github.com/ljh-sh/bpan/internal/sandbox"
	"github.com/ljh-sh/bpan/internal/version"
)

// globalFlags are parsed before dispatch and shared by all subcommands.
type globalFlags struct {
	configPath string
	jsonOut    bool
	verbose    bool
	noColor    bool
}

// appError is a structured error printed to stderr (and JSON if --json).
type appError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
	Hint    string `json:"hint,omitempty"`
}

func (e *appError) Error() string {
	if e.Hint != "" {
		return fmt.Sprintf("bpan: %s\n  hint: %s", e.Message, e.Hint)
	}
	return "bpan: " + e.Message
}

func main() {
	// SIGINT handling — cancel in-flight OAuth polls cleanly.
	ctx, cancel := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer cancel()

	if len(os.Args) < 2 {
		usage()
		os.Exit(2)
	}
	if os.Args[1] == "-h" || os.Args[1] == "--help" {
		usage()
		return
	}
	if os.Args[1] == "version" {
		cmdVersion()
		return
	}
	if os.Args[1] == "help" {
		if len(os.Args) >= 3 {
			helpSubcommand(os.Args[2])
		} else {
			usage()
		}
		return
	}

	g, rest, err := parseGlobalFlags(os.Args[1:])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(2)
	}

	cmd := ""
	if len(rest) > 0 {
		cmd = rest[0]
	}

	// Resolve default config path lazily.
	if g.configPath == "" {
		g.configPath, err = config.DefaultPath()
		if err != nil {
			fail(g, 1, err.Error(), "set BDPAN_CONFIG or pass --config")
		}
	}

	switch cmd {
	case "":
		usage()
		os.Exit(2)
	case "login":
		cmdLogin(ctx, g, rest[1:])
	case "logout":
		cmdLogout(g)
	case "whoami":
		cmdWhoami(ctx, g)
	case "ls":
		cmdLs(ctx, g, rest[1:])
	case "upload":
		cmdUpload(ctx, g, rest[1:])
	case "download":
		cmdDownload(ctx, g, rest[1:])
	case "search":
		cmdSearch(ctx, g, rest[1:])
	case "quota":
		cmdQuota(ctx, g)
	case "mkdir":
		cmdMkdir(ctx, g, rest[1:])
	case "rm":
		cmdRm(ctx, g, rest[1:])
	case "mv":
		cmdMv(ctx, g, rest[1:])
	case "cp":
		cmdCp(ctx, g, rest[1:])
	case "rename":
		cmdRename(ctx, g, rest[1:])
	case "install":
		cmdInstall(g)
	case "update":
		cmdUpdate()
	case "uninstall":
		cmdUninstall()
	case "transfer", "share":
		fail(g, 1, "not yet implemented — coming in v0.2.0", "")
	default:
		fmt.Fprintf(os.Stderr, "bpan: unknown command %q\n", cmd)
		usage()
		os.Exit(2)
	}
}

func parseGlobalFlags(args []string) (g globalFlags, rest []string, err error) {
	fs := flag.NewFlagSet("bpan", flag.ContinueOnError)
	fs.SetOutput(io.Discard) // we print our own errors
	fs.StringVar(&g.configPath, "config", "", "config file path (default ~/.config/bdpan/config.json)")
	fs.BoolVar(&g.jsonOut, "json", false, "JSON output for machine-readable responses")
	fs.BoolVar(&g.verbose, "verbose", false, "verbose logging")
	fs.BoolVar(&g.noColor, "no-color", false, "disable ANSI color output")
	if err := fs.Parse(args); err != nil {
		return g, nil, fmt.Errorf("bpan: %w", err)
	}
	return g, fs.Args(), nil
}

// fail exits with a structured error.
func fail(g globalFlags, code int, msg, hint string) {
	if g.jsonOut {
		_ = json.NewEncoder(os.Stdout).Encode(&appError{Code: code, Message: msg, Hint: hint})
	} else {
		e := &appError{Code: code, Message: msg, Hint: hint}
		fmt.Fprintln(os.Stderr, e.Error())
	}
	os.Exit(code)
}

func usage() {
	fmt.Println(`bpan — Baidu Netdisk CLI

Usage:
  bpan [global flags] <command> [command flags] [args...]

Global flags:
  --config <path>   config file path (default ~/.config/bdpan/config.json)
  --json            output JSON for machine-readable responses
  --verbose         verbose logging
  --no-color        disable ANSI color output

Commands:
  login              OAuth device-code login
  logout             delete saved credentials
  whoami             show the logged-in Baidu account
  ls [path]          list a Netdisk directory
  upload <l> <r>     upload a local file to Netdisk
  download <r> <l>   download a Netdisk file to local
  search <query>     semantic search on Netdisk
  quota              show Netdisk storage usage
  mkdir <path>       create a Netdisk directory
  rm <path>          delete a Netdisk file or directory
  mv <src> <dst>     move or rename a Netdisk file
  cp <src> <dst>     copy a Netdisk file
  rename <p> <name>  rename a Netdisk file in place
  install            install bpan to ~/.local/bin/
  update             self-update via GitHub Releases
  uninstall          remove installed binary
  version            print version and exit
  help [command]     print command-specific help

Environment:
  BDPAN_CLIENT_ID       Baidu Open Platform AppKey (required for login)
  BDPAN_CLIENT_SECRET   Baidu Open Platform SecretKey (required for login/refresh)
  BDPAN_CONFIG          override --config path

Run 'bpan help <command>' for flag details on a specific command.`)
}

func helpSubcommand(cmd string) {
	help, ok := helpText[cmd]
	if !ok {
		fmt.Fprintf(os.Stderr, "bpan: no help for %q\n", cmd)
		os.Exit(2)
	}
	fmt.Println(help)
}

var helpText = map[string]string{
	"login": `bpan login — OAuth device-code login

Walks you through Baidu's RFC 8628 device-code flow. Prints a URL and a short
user_code; you open the URL, enter the code, and bpan picks up the resulting
tokens.

Required environment:
  BDPAN_CLIENT_ID       Baidu Open Platform AppKey
  BDPAN_CLIENT_SECRET   Baidu Open Platform SecretKey

Apply at https://pan.baidu.com/union/ to obtain a personal AppKey.`,
	"ls": `bpan ls [path] — list a Netdisk directory

Flags:
  --limit N    limit number of entries (default 50)
  --order time|name|size   sort order (default time)
  --desc       sort descending

Path defaults to "/" (Netdisk root).`,
	"upload": `bpan upload <local> <remote> — upload a file

  <local>    path to a local file (required)
  <remote>   absolute Netdisk path including filename (required)

Flags:
  --overwrite         overwrite the destination if it exists
  --chunk-size MB     slice size in MB (default 4)`,
	"download": `bpan download <remote> <local> — download a file

  <remote>   absolute Netdisk path (required)
  <local>    local destination path (required)`,
	"search": `bpan search <query> — semantic search

Flags:
  --dir PATH    restrict search to a directory (default "/")
  --type file|dir|all   restrict result type (default all)`,
	"quota": `bpan quota — show Netdisk storage usage`,
	"install": `bpan install — copy bpan into ~/.local/bin/bpan

Symlinks the running binary so subsequent releases can 'bpan update' in place.`,
}

// ── Subcommand implementations ────────────────────────────────────────────────

func cmdVersion() {
	fmt.Println(version.String())
	fmt.Println("Powered by Baidu Netdisk Open Platform")
}

func cmdLogin(ctx context.Context, g globalFlags, args []string) {
	appKey := os.Getenv("BDPAN_CLIENT_ID")
	appSecret := os.Getenv("BDPAN_CLIENT_SECRET")
	if appKey == "" || appSecret == "" {
		fail(g, 2,
			"BDPAN_CLIENT_ID and BDPAN_CLIENT_SECRET are required for login",
			"apply for a personal AppKey at https://pan.baidu.com/union/")
	}

	cfg, err := config.Load(g.configPath)
	if err != nil {
		fail(g, 1, err.Error(), "")
	}
	if err := auth.Login(ctx, appKey, appSecret, cfg, g.configPath); err != nil {
		fail(g, 1, err.Error(), "")
	}
}

func cmdLogout(g globalFlags) {
	if err := config.Delete(g.configPath); err != nil {
		fail(g, 1, err.Error(), "")
	}
	fmt.Println("✓ Logged out — credentials deleted.")
}

func mustClient(ctx context.Context, g globalFlags) *client.Client {
	appKey := os.Getenv("BDPAN_CLIENT_ID")
	appSecret := os.Getenv("BDPAN_CLIENT_SECRET")
	cfg, err := config.Load(g.configPath)
	if err != nil {
		fail(g, 1, err.Error(), "")
	}
	c, err := client.NewWith(ctx, cfg, g.configPath, appKey, appSecret)
	if err != nil {
		fail(g, 3, err.Error(), "run 'bpan login' to authenticate")
	}
	return c
}

func cmdWhoami(ctx context.Context, g globalFlags) {
	c := mustClient(ctx, g)
	info, err := c.Scene().UserInfo(ctx)
	if err != nil {
		fail(g, 1, "whoami: "+err.Error(), "")
	}
	if g.jsonOut {
		_ = json.NewEncoder(os.Stdout).Encode(info)
		return
	}
	vip := []string{"normal", "member", "svip"}[info.VipType]
	if info.VipType < 0 || info.VipType > 2 {
		vip = strconv.Itoa(info.VipType)
	}
	fmt.Printf("Baidu name: %s\n", info.BaiduName)
	fmt.Printf("Netdisk:    %s\n", info.NetdiskName)
	fmt.Printf("UK:         %d\n", info.UK)
	fmt.Printf("VIP type:   %s\n", vip)
	if info.AvatarURL != "" {
		fmt.Printf("Avatar:     %s\n", info.AvatarURL)
	}
	fmt.Println()
	fmt.Println("Powered by Baidu Netdisk Open Platform")
}

func cmdLs(ctx context.Context, g globalFlags, args []string) {
	fs := flag.NewFlagSet("ls", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	var (
		limit = fs.Int("limit", 50, "max entries")
		order = fs.String("order", "time", "sort field: time|name|size")
		desc  = fs.Bool("desc", false, "sort descending")
	)
	if err := fs.Parse(args); err != nil {
		fail(g, 2, "ls: "+err.Error(), "")
	}
	path := "/"
	if fs.NArg() > 0 {
		path = fs.Arg(0)
	}
	np, err := sandbox.NormalizeRemotePath(path)
	if err != nil {
		fail(g, 2, err.Error(), "")
	}

	c := mustClient(ctx, g)
	files, err := c.Scene().ListDir(ctx, np, &scene.ListDirOptions{
		Order: *order, Desc: *desc, Limit: *limit,
	})
	if err != nil {
		fail(g, 1, "ls: "+err.Error(), "")
	}

	if g.jsonOut {
		_ = json.NewEncoder(os.Stdout).Encode(map[string]any{
			"path":    np,
			"entries": files,
			"total":   len(files),
		})
		return
	}
	if len(files) == 0 {
		fmt.Printf("(empty directory %s)\n", np)
		return
	}
	tw := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	for _, f := range files {
		kind := "-"
		if f.IsDir {
			kind = "d"
		}
		mtime := time.Unix(f.Mtime, 0).Format("2006-01-02 15:04")
		fmt.Fprintf(tw, "%s\t%s\t%s\t%s\n",
			kind,
			humanSize(f.Size),
			mtime,
			f.Filename,
		)
	}
	tw.Flush()
	fmt.Printf("total: %d entries\n", len(files))
}

func humanSize(n int64) string {
	const k = 1024
	if n < k {
		return fmt.Sprintf("%d B", n)
	}
	div, exp := int64(k), 0
	for x := n / k; x >= k; x /= k {
		div *= k
		exp++
	}
	return fmt.Sprintf("%.1f %cB", float64(n)/float64(div), "KMGT"[exp])
}

func cmdUpload(ctx context.Context, g globalFlags, args []string) {
	fs := flag.NewFlagSet("upload", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	var (
		overwrite  = fs.Bool("overwrite", false, "overwrite destination if it exists")
		chunkSize  = fs.Int("chunk-size", 4, "slice size in MB (default 4)")
	)
	if err := fs.Parse(args); err != nil {
		fail(g, 2, "upload: "+err.Error(), "")
	}
	if fs.NArg() != 2 {
		fail(g, 2, "upload requires <local> <remote>", "")
	}
	local, remote := fs.Arg(0), fs.Arg(1)
	rp, err := sandbox.NormalizeRemotePath(remote)
	if err != nil {
		fail(g, 2, err.Error(), "")
	}

	c := mustClient(ctx, g)
	res, err := c.Scene().UploadFile(ctx, &scene.UploadFileParams{
		LocalPath:  local,
		RemotePath: rp,
		SliceSize:  int64(*chunkSize) * 1024 * 1024,
		RType:      rtypeForOverwrite(*overwrite),
	})
	if err != nil {
		fail(g, 1, "upload: "+err.Error(), "")
	}
	if g.jsonOut {
		_ = json.NewEncoder(os.Stdout).Encode(res)
		return
	}
	fmt.Printf("✓ uploaded %s → %s\n", local, rp)
}

func rtypeForOverwrite(overwrite bool) *int {
	v := 1 // default: fail on conflict
	if overwrite {
		v = 3 // overwrite
	}
	return api.Ptr(v)
}

func cmdDownload(ctx context.Context, g globalFlags, args []string) {
	if len(args) != 2 {
		fail(g, 2, "download requires <remote> <local>", "")
	}
	rp, err := sandbox.NormalizeRemotePath(args[0])
	if err != nil {
		fail(g, 2, err.Error(), "")
	}
	local := args[1]

	c := mustClient(ctx, g)
	// List parent dir to resolve filename → fs_id, then call DownloadFile.
	parent := parentDir(rp)
	files, err := c.Scene().ListDir(ctx, parent, &scene.ListDirOptions{Limit: 1000})
	if err != nil {
		fail(g, 1, "download (list parent): "+err.Error(), "")
	}
	name := strings.TrimPrefix(rp, parent+"/")
	if name == rp {
		name = strings.TrimPrefix(rp, parent)
	}
	var fsID int64
	for _, f := range files {
		if f.Filename == name {
			fsID = f.FsID
			break
		}
	}
	if fsID == 0 {
		fail(g, 5, fmt.Sprintf("remote file not found: %s", rp), "")
	}
	res, err := c.Scene().DownloadFile(ctx, &scene.DownloadFileParams{
		FsID: fsID, LocalPath: local,
	})
	if err != nil {
		fail(g, 1, "download: "+err.Error(), "")
	}
	if g.jsonOut {
		_ = json.NewEncoder(os.Stdout).Encode(res)
		return
	}
	fmt.Printf("✓ downloaded %s → %s\n", rp, local)
}

func parentDir(p string) string {
	if p == "/" {
		return "/"
	}
	if i := strings.LastIndex(p, "/"); i > 0 {
		return p[:i]
	}
	return "/"
}

func cmdSearch(ctx context.Context, g globalFlags, args []string) {
	fs := flag.NewFlagSet("search", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	var (
		dir  = fs.String("dir", "/", "restrict search to a directory")
		typ  = fs.String("type", "all", "result type: file|dir|all")
	)
	if err := fs.Parse(args); err != nil {
		fail(g, 2, "search: "+err.Error(), "")
	}
	if fs.NArg() < 1 {
		fail(g, 2, "search requires <query>", "")
	}
	np, err := sandbox.NormalizeRemotePath(*dir)
	if err != nil {
		fail(g, 2, err.Error(), "")
	}

	c := mustClient(ctx, g)
	var category []int
	switch *typ {
	case "file":
		category = []int{1}
	case "dir":
		category = []int{0}
	default:
		category = nil
	}
	res, err := c.Scene().Search(ctx, &scene.SearchParams{
		Query: fs.Arg(0), Dir: np, Category: category,
	})
	if err != nil {
		fail(g, 1, "search: "+err.Error(), "")
	}
	if g.jsonOut {
		_ = json.NewEncoder(os.Stdout).Encode(res)
		return
	}
	for _, r := range res {
		fmt.Printf("%s\n", r.Filename)
	}
}

func cmdQuota(ctx context.Context, g globalFlags) {
	c := mustClient(ctx, g)
	q, err := c.API().Nas.Quota(ctx, &api.QuotaParams{})
	if err != nil {
		fail(g, 1, "quota: "+err.Error(), "")
	}
	if g.jsonOut {
		_ = json.NewEncoder(os.Stdout).Encode(q)
		return
	}
	fmt.Printf("Quota:     %s / %s\n", humanSize(q.Used), humanSize(q.Total))
	fmt.Printf("Free:      %s\n", humanSize(q.Free))
}

func cmdMkdir(ctx context.Context, g globalFlags, args []string) {
	if len(args) != 1 {
		fail(g, 2, "mkdir requires <path>", "")
	}
	p, err := sandbox.NormalizeRemotePath(args[0])
	if err != nil {
		fail(g, 2, err.Error(), "")
	}
	c := mustClient(ctx, g)
	if err := c.Scene().MkdirIfNotExist(ctx, p); err != nil {
		fail(g, 1, "mkdir: "+err.Error(), "")
	}
	fmt.Printf("✓ mkdir %s\n", p)
}

func cmdRm(ctx context.Context, g globalFlags, args []string) {
	if len(args) != 1 {
		fail(g, 2, "rm requires <path>", "")
	}
	p, err := sandbox.NormalizeRemotePath(args[0])
	if err != nil {
		fail(g, 2, err.Error(), "")
	}
	c := mustClient(ctx, g)
	if err := c.Scene().DeleteFile(ctx, []string{p}); err != nil {
		fail(g, 1, "rm: "+err.Error(), "")
	}
	fmt.Printf("✓ rm %s\n", p)
}

func cmdMv(ctx context.Context, g globalFlags, args []string) {
	if len(args) != 2 {
		fail(g, 2, "mv requires <src> <dst>", "")
	}
	src, err := sandbox.NormalizeRemotePath(args[0])
	if err != nil {
		fail(g, 2, err.Error(), "")
	}
	dst, err := sandbox.NormalizeRemotePath(args[1])
	if err != nil {
		fail(g, 2, err.Error(), "")
	}
	dstDir := parentDir(dst)
	newName := strings.TrimPrefix(dst, dstDir+"/")
	if newName == dst {
		newName = strings.TrimPrefix(dst, dstDir)
	}
	c := mustClient(ctx, g)
	if err := c.Scene().MoveFile(ctx, src, dstDir, newName); err != nil {
		fail(g, 1, "mv: "+err.Error(), "")
	}
	fmt.Printf("✓ mv %s → %s\n", src, dst)
}

func cmdCp(ctx context.Context, g globalFlags, args []string) {
	if len(args) != 2 {
		fail(g, 2, "cp requires <src> <dst>", "")
	}
	src, err := sandbox.NormalizeRemotePath(args[0])
	if err != nil {
		fail(g, 2, err.Error(), "")
	}
	dst, err := sandbox.NormalizeRemotePath(args[1])
	if err != nil {
		fail(g, 2, err.Error(), "")
	}
	dstDir := parentDir(dst)
	newName := strings.TrimPrefix(dst, dstDir+"/")
	if newName == dst {
		newName = strings.TrimPrefix(dst, dstDir)
	}
	c := mustClient(ctx, g)
	if err := c.Scene().CopyFile(ctx, src, dstDir, newName); err != nil {
		fail(g, 1, "cp: "+err.Error(), "")
	}
	fmt.Printf("✓ cp %s → %s\n", src, dst)
}

func cmdRename(ctx context.Context, g globalFlags, args []string) {
	if len(args) != 2 {
		fail(g, 2, "rename requires <path> <newname>", "")
	}
	p, err := sandbox.NormalizeRemotePath(args[0])
	if err != nil {
		fail(g, 2, err.Error(), "")
	}
	c := mustClient(ctx, g)
	if err := c.Scene().RenameFile(ctx, p, args[1]); err != nil {
		fail(g, 1, "rename: "+err.Error(), "")
	}
	fmt.Printf("✓ rename %s → %s\n", p, args[1])
}

// cmdInstall symlinks the running binary to ~/.local/bin/bpan so the user
// can type 'bpan' on PATH. It does not modify anything inside the source tree.
func cmdInstall(g globalFlags) {
	exe, err := os.Executable()
	if err != nil {
		fail(g, 1, "install: cannot determine current executable: "+err.Error(), "")
	}
	home, err := os.UserHomeDir()
	if err != nil {
		fail(g, 1, "install: cannot determine home: "+err.Error(), "")
	}
	dst := filepath.Join(home, ".local", "bin", "bpan")
	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		fail(g, 1, "install: mkdir: "+err.Error(), "")
	}
	// Remove existing file/symlink at dst before linking.
	_ = os.Remove(dst)
	if err := os.Symlink(exe, dst); err != nil {
		// If symlink fails (e.g. cross-device on Windows), fall back to copy.
		data, rerr := os.ReadFile(exe)
		if rerr != nil {
			fail(g, 1, "install: symlink and read fallback failed: "+err.Error(), "")
		}
		if werr := os.WriteFile(dst, data, 0o755); werr != nil {
			fail(g, 1, "install: symlink/copy failed: "+err.Error(), "")
		}
	}
	fmt.Printf("✓ installed bpan → %s\n", dst)
	fmt.Printf("  (add %s to PATH if not already)\n", filepath.Dir(dst))
}

// cmdUpdate is a placeholder for v0.1.0 — full self-update via GitHub Releases
// lands in v0.1.1 once the release workflow has produced at least one tagged
// artifact we can compare against.
func cmdUpdate() {
	fmt.Println("bpan: update is not yet implemented — install v0.2.0 manually for now.")
	fmt.Println("      see https://github.com/ljh-sh/bpan/releases")
	os.Exit(1)
}

// cmdUninstall removes the symlink/binary that cmdInstall created.
func cmdUninstall() {
	home, err := os.UserHomeDir()
	if err != nil {
		fmt.Fprintln(os.Stderr, "bpan: cannot determine home:", err)
		os.Exit(1)
	}
	dst := filepath.Join(home, ".local", "bin", "bpan")
	if err := os.Remove(dst); err != nil && !errors.Is(err, os.ErrNotExist) {
		fmt.Fprintln(os.Stderr, "bpan: uninstall:", err)
		os.Exit(1)
	}
	fmt.Printf("✓ removed %s\n", dst)
}

// Reference: keep these imports referenced so go vet doesn't complain when
// tree-shaking unused imports.
var _ = time.RFC3339