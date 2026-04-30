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
		"targetLegalityChecker := BuildTargetLegalityChecker(state)",
	}

	for _, marker := range forbiddenMarkers {
		if bytes.Contains(content, []byte(marker)) {
			t.Fatalf("engine orchestration guard violated: found %q in engine.go; extract permission helpers to dedicated module", marker)
		}
	}
}

func TestEngineOrchestrationGuard_NoGenericPreflightChecks(t *testing.T) {
	path := filepath.Join(mustRulesPackageDirForEngineGuard(t), "engine.go")
	content, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("os.ReadFile(%q) returned error: %v", path, err)
	}

	forbiddenMarkers := []string{
		"if actionRequiresPriority(action.Kind) && action.ActorID != currentPriorityPlayerID(state) {",
		"if actionRequiresEmptyStack(action.Kind) && len(state.Board.Stack) != 0 {",
		"if action.TargetPlayerID != \"\" && !containsString(state.Players, action.TargetPlayerID) {",
		"if action.TargetCardID != \"\" && !hasCardID(state, action.TargetCardID) {",
	}

	for _, marker := range forbiddenMarkers {
		if bytes.Contains(content, []byte(marker)) {
			t.Fatalf("engine orchestration guard violated: found %q in engine.go; extract generic preflight checks to dedicated module", marker)
		}
	}
}

func TestEngineOrchestrationGuard_NoCommitStateOrHistoryWrites(t *testing.T) {
	path := filepath.Join(mustRulesPackageDirForEngineGuard(t), "engine.go")
	content, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("os.ReadFile(%q) returned error: %v", path, err)
	}

	forbiddenMarkers := []string{
		"func commitState(",
		"committed.History.Actions = append(",
		"committed.History.Operations = append(",
		"committed.History.Events = append(",
		"committed.History.Revisions = append(",
		"committed.Match.FinishedAtRevision =",
	}

	for _, marker := range forbiddenMarkers {
		if bytes.Contains(content, []byte(marker)) {
			t.Fatalf("engine orchestration guard violated: found %q in engine.go; extract commit/history writes to submit pipeline or transition helpers", marker)
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
