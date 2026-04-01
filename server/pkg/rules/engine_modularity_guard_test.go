package rules

import (
	"bytes"
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

func TestEngineOrchestrationGuard_NoActionPermissionHelpers(t *testing.T) {
	path := filepath.Join(mustRulesPackageDirForEngineGuard(t), "engine.go")
	content, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("os.ReadFile(%q) returned error: %v", path, err)
	}

	forbiddenMarkers := []string{
		"func checkCardActionPermissionLegality(",
		"func permissionForActionKind(",
	}

	for _, marker := range forbiddenMarkers {
		if bytes.Contains(content, []byte(marker)) {
			t.Fatalf("engine orchestration guard violated: found %q in engine.go; extract permission helpers to dedicated module", marker)
		}
	}
}

func mustRulesPackageDirForEngineGuard(t *testing.T) string {
	t.Helper()
	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("runtime.Caller(0) failed")
	}
	return filepath.Dir(currentFile)
}
