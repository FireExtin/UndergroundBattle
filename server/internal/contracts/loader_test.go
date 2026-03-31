package contracts

import (
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
	"slices"
	"testing"

	contractspkg "undergroundbattle/server/pkg/contracts"
)

// TestContractFixturesMatchExpectations proves Go consumes the same CardLogic fixtures as the TypeScript toolchain.
func TestContractFixturesMatchExpectations(t *testing.T) {
	currentVersion := currentCardSchemaVersion(t)

	for _, fixturePath := range fixturePaths(t) {
		t.Run(filepath.Base(fixturePath), func(t *testing.T) {
			fixture, err := LoadFixture(fixturePath)
			if err != nil {
				t.Fatalf("LoadFixture returned error: %v", err)
			}

			if fixture.SchemaVersion != currentVersion {
				t.Fatalf("fixture schemaVersion = %q, want %q", fixture.SchemaVersion, currentVersion)
			}

			if fixture.Card.Name == "" {
				t.Fatal("fixture card.name must not be empty")
			}

			if fixture.Card.SourcePath == "" {
				t.Fatal("fixture card.sourcePath must not be empty")
			}

			if fixture.Input.Logic.SchemaVersion != currentVersion {
				t.Fatalf("logic schemaVersion = %q, want %q", fixture.Input.Logic.SchemaVersion, currentVersion)
			}

			if !fixture.Expectations.ParseOK {
				t.Fatal("parseOk must stay true for committed contract fixtures")
			}

			parsed := contractspkg.ParseFixtureLogic(fixture)

			if parsed.Speed != fixture.Expectations.Speed {
				t.Fatalf("speed = %q, want %q", parsed.Speed, fixture.Expectations.Speed)
			}

			if !slices.Equal(parsed.TargetKinds, fixture.Expectations.TargetKinds) {
				t.Fatalf("targetKinds = %v, want %v", parsed.TargetKinds, fixture.Expectations.TargetKinds)
			}

			if parsed.RequiresStack != fixture.Expectations.RequiresStack {
				t.Fatalf("requiresStack = %v, want %v", parsed.RequiresStack, fixture.Expectations.RequiresStack)
			}

			if parsed.DurationKind != fixture.Expectations.DurationKind {
				t.Fatalf("durationKind = %q, want %q", parsed.DurationKind, fixture.Expectations.DurationKind)
			}

			if optionalString(parsed.ScriptID) != optionalString(fixture.Expectations.ScriptID) {
				t.Fatalf("scriptId = %q, want %q", optionalString(parsed.ScriptID), optionalString(fixture.Expectations.ScriptID))
			}

			if parsed.CardName != fixture.Card.Name {
				t.Fatalf("cardName = %q, want %q", parsed.CardName, fixture.Card.Name)
			}

			if parsed.SourcePath != fixture.Card.SourcePath {
				t.Fatalf("sourcePath = %q, want %q", parsed.SourcePath, fixture.Card.SourcePath)
			}

			if fixture.Expectations.ScriptID != nil {
				if !parsed.RequiresScript {
					t.Fatal("scripted fixture must set requiresScript to true")
				}

				if parsed.PureDSLExecutable {
					t.Fatal("scripted fixture must not be treated as pure DSL executable")
				}
			} else {
				if parsed.RequiresScript {
					t.Fatal("pure DSL fixture unexpectedly requires a script")
				}

				if !parsed.PureDSLExecutable {
					t.Fatal("pure DSL fixture should remain executable without a script")
				}
			}
		})
	}
}

func TestDefaultFixtureCatalogIndexesRealCards(t *testing.T) {
	catalog, err := LoadDefaultFixtureCatalog()
	if err != nil {
		t.Fatalf("LoadDefaultFixtureCatalog returned error: %v", err)
	}

	if catalog.Len() != 10 {
		t.Fatalf("catalog length = %d, want 10", catalog.Len())
	}

	fixture, ok := catalog.Find("BQ010")
	if !ok {
		t.Fatal("expected BQ010 fixture in catalog")
	}

	if fixture.Card.Name != "读心术" {
		t.Fatalf("fixture card name = %q, want %q", fixture.Card.Name, "读心术")
	}

	if fixture.Card.SourcePath != "organized_content/cards/事/cards.json" {
		t.Fatalf("fixture sourcePath = %q, want %q", fixture.Card.SourcePath, "organized_content/cards/事/cards.json")
	}

	newFixture, ok := catalog.Find("WM090")
	if !ok {
		t.Fatal("expected WM090 fixture in catalog")
	}

	if newFixture.Card.Name != "茶叶占卜法" {
		t.Fatalf("fixture card name = %q, want %q", newFixture.Card.Name, "茶叶占卜法")
	}
}

func fixturePaths(t *testing.T) []string {
	t.Helper()

	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("runtime.Caller failed")
	}

	paths, err := filepath.Glob(filepath.Clean(filepath.Join(filepath.Dir(currentFile), "../../../shared/contracts/fixtures/*.fixture.json")))
	if err != nil {
		t.Fatalf("filepath.Glob returned error: %v", err)
	}

	if len(paths) == 0 {
		t.Fatal("expected at least one contract fixture")
	}

	return paths
}

func currentCardSchemaVersion(t *testing.T) string {
	t.Helper()

	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("runtime.Caller failed")
	}

	schemaPath := filepath.Clean(filepath.Join(filepath.Dir(currentFile), "../../../shared/schemas/card.schema.json"))
	data, err := os.ReadFile(schemaPath)
	if err != nil {
		t.Fatalf("os.ReadFile(%q) returned error: %v", schemaPath, err)
	}

	var payload struct {
		Current string `json:"x-currentSchemaVersion"`
	}

	if err := json.Unmarshal(data, &payload); err != nil {
		t.Fatalf("json.Unmarshal returned error: %v", err)
	}

	if payload.Current == "" {
		t.Fatal("x-currentSchemaVersion must be declared in shared/schemas/card.schema.json")
	}

	return payload.Current
}

func optionalString(value *string) string {
	if value == nil {
		return ""
	}

	return *value
}
