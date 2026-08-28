package sandbox

import "testing"

func TestNormalizeRemotePath(t *testing.T) {
	cases := []struct {
		in, want string
		wantErr  bool
	}{
		{"", "/", false},
		{"/", "/", false},
		{"/foo", "/foo", false},
		{"foo", "/foo", false},
		{"/foo/bar/", "/foo/bar", false},
		{"/foo/./bar", "/foo/bar", false},
		{"/foo/../bar", "/bar", false},
		{"/foo/../../bar", "", true}, // escapes root
		{"/foo\\bar", "", true},      // backslash rejected
		{"/foo\x00bar", "", true},    // NUL rejected
		{"/foo\nbar", "", true},      // newline rejected
		{"//", "/", false},
		{"/./", "/", false},
		{"//foo//bar//", "/foo/bar", false},
	}
	for _, c := range cases {
		got, err := NormalizeRemotePath(c.in)
		if (err != nil) != c.wantErr {
			t.Errorf("NormalizeRemotePath(%q) err=%v wantErr=%v", c.in, err, c.wantErr)
			continue
		}
		if !c.wantErr && got != c.want {
			t.Errorf("NormalizeRemotePath(%q)=%q want %q", c.in, got, c.want)
		}
	}
}

func TestJoinRemote(t *testing.T) {
	got, err := JoinRemote("/a/b", "c")
	if err != nil {
		t.Fatal(err)
	}
	if got != "/a/b/c" {
		t.Errorf("JoinRemote=/a/b/c/c want /a/b/c got %s", got)
	}
	got, err = JoinRemote("/", "/abs")
	if err != nil {
		t.Fatal(err)
	}
	if got != "/abs" {
		t.Errorf("JoinRemote / /abs want /abs got %s", got)
	}
}