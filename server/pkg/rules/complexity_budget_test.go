package rules

import (
	"bufio"
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

type fileComplexityBudget struct {
	File     string
	MaxLines int
}

var coreRulesComplexityBudgets = []fileComplexityBudget{
	{File: "engine.go", MaxLines: 1250},
	{File: "types.go", MaxLines: 730},
	{File: "continuous.go", MaxLines: 450},
	{File: "attachment.go", MaxLines: 280},
	{File: "projection.go", MaxLines: 340},
}

func TestCoreRulesComplexityBudgets(t *testing.T) {
	rulesDir := mustRulesPackageDir(t)

	for _, budget := range coreRulesComplexityBudgets {
		budget := budget
		t.Run(budget.File, func(t *testing.T) {
			lineCount, err := countFileLines(filepath.Join(rulesDir, budget.File))
			if err != nil {
				t.Fatalf("countFileLines returned error: %v", err)
			}

			if lineCount > budget.MaxLines {
				t.Fatalf(
					"complexity budget exceeded for %s: %d lines > %d; extract module boundaries before adding more logic",
					budget.File,
					lineCount,
					budget.MaxLines,
				)
			}
		})
	}
}

func mustRulesPackageDir(t *testing.T) string {
	t.Helper()
	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("runtime.Caller(0) failed")
	}
	return filepath.Dir(currentFile)
}

func countFileLines(path string) (int, error) {
	file, err := os.Open(path)
	if err != nil {
		return 0, err
	}
	defer file.Close()

	lineCount := 0
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		lineCount++
	}
	if err := scanner.Err(); err != nil {
		return 0, err
	}

	return lineCount, nil
}
