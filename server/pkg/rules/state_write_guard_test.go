package rules

import (
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"strings"
	"testing"
)

type stateWriteGuard struct {
	name          string
	pattern       *regexp.Regexp
	allowedFiles  map[string]bool
	failureReason string
}

func TestStateTransitionWriteGuardsForCriticalFields(t *testing.T) {
	rulesDir := mustRulesPackageDirForWriteGuard(t)
	sourceFiles := mustRulesSourceFiles(t, rulesDir)

	guards := []stateWriteGuard{
		{
			name:          "face_down_assignment",
			pattern:       regexp.MustCompile(`\.Board\.Cards\[[^\]]+\]\.FaceDown\s*=`),
			allowedFiles:  map[string]bool{"state_transitions.go": true},
			failureReason: "direct FaceDown writes must go through transition helpers",
		},
		{
			name:          "revealed_assignment",
			pattern:       regexp.MustCompile(`\.Board\.Cards\[[^\]]+\]\.Revealed\s*=`),
			allowedFiles:  map[string]bool{"state_transitions.go": true},
			failureReason: "direct Revealed writes must go through transition helpers",
		},
		{
			name:          "marker_registry_write",
			pattern:       regexp.MustCompile(`\.Board\.Markers\.SetMarker\(`),
			allowedFiles:  map[string]bool{"state_transitions.go": true},
			failureReason: "marker writes must go through setMarker/addMarkerCount/removeMarkerCount transitions",
		},
		{
			name:          "resolved_append",
			pattern:       regexp.MustCompile(`\.Board\.Resolved\s*=\s*append\(`),
			allowedFiles:  map[string]bool{"state_transitions.go": true},
			failureReason: "resolved operation writes must go through appendResolvedOperation transition",
		},
		{
			name:          "random_results_append",
			pattern:       regexp.MustCompile(`\.Board\.RandomResults\s*=\s*append\(`),
			allowedFiles:  map[string]bool{"state_transitions.go": true},
			failureReason: "random results writes must go through appendRandomResult transition",
		},
	}

	for _, guard := range guards {
		guard := guard
		t.Run(guard.name, func(t *testing.T) {
			for _, path := range sourceFiles {
				base := filepath.Base(path)
				if guard.allowedFiles[base] {
					continue
				}

				content, err := os.ReadFile(path)
				if err != nil {
					t.Fatalf("os.ReadFile(%q) returned error: %v", path, err)
				}

				if guard.pattern.Match(content) {
					t.Fatalf("%s (%s): %s", path, guard.pattern.String(), guard.failureReason)
				}
			}
		})
	}
}

func mustRulesSourceFiles(t *testing.T, rulesDir string) []string {
	t.Helper()

	entries, err := os.ReadDir(rulesDir)
	if err != nil {
		t.Fatalf("os.ReadDir(%q) returned error: %v", rulesDir, err)
	}

	files := make([]string, 0, len(entries))
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		name := entry.Name()
		if !strings.HasSuffix(name, ".go") || strings.HasSuffix(name, "_test.go") {
			continue
		}
		files = append(files, filepath.Join(rulesDir, name))
	}

	return files
}

func mustRulesPackageDirForWriteGuard(t *testing.T) string {
	t.Helper()
	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("runtime.Caller(0) failed")
	}
	return filepath.Dir(currentFile)
}
