package contracts

import (
	"fmt"
	"path/filepath"
	"runtime"
	"slices"
	"sync"

	contractspkg "undergroundbattle/server/pkg/contracts"
)

// Purpose: Loads and caches the shared CardLogic fixture catalog so the Go rules kernel can resolve real card entry points.

// FixtureCatalog indexes fixtures by cardId for fast rules-kernel lookup.
type FixtureCatalog struct {
	byCardID map[string]contractspkg.Fixture
}

var (
	defaultFixtureCatalogOnce sync.Once
	defaultFixtureCatalog     FixtureCatalog
	defaultFixtureCatalogErr  error
)

// LoadDefaultFixtureCatalog returns the repository fixture catalog cached for the current process.
func LoadDefaultFixtureCatalog() (FixtureCatalog, error) {
	defaultFixtureCatalogOnce.Do(func() {
		dir, err := defaultFixtureDirectory()
		if err != nil {
			defaultFixtureCatalogErr = err
			return
		}

		defaultFixtureCatalog, defaultFixtureCatalogErr = LoadFixtureCatalog(dir)
	})

	return defaultFixtureCatalog.clone(), defaultFixtureCatalogErr
}

// LoadFixtureCatalog reads all committed fixture JSON files under one directory.
func LoadFixtureCatalog(dir string) (FixtureCatalog, error) {
	paths, err := filepath.Glob(filepath.Join(dir, "*.fixture.json"))
	if err != nil {
		return FixtureCatalog{}, err
	}

	if len(paths) == 0 {
		return FixtureCatalog{}, fmt.Errorf("no fixture files found under %s", dir)
	}

	slices.Sort(paths)
	byCardID := make(map[string]contractspkg.Fixture, len(paths))
	for _, fixturePath := range paths {
		fixture, err := LoadFixture(fixturePath)
		if err != nil {
			return FixtureCatalog{}, err
		}

		if _, exists := byCardID[fixture.CardID]; exists {
			return FixtureCatalog{}, fmt.Errorf("duplicate fixture cardId %q", fixture.CardID)
		}

		byCardID[fixture.CardID] = cloneFixture(fixture)
	}

	return FixtureCatalog{byCardID: byCardID}, nil
}

// Find returns one fixture by cardId.
func (catalog FixtureCatalog) Find(cardID string) (contractspkg.Fixture, bool) {
	fixture, ok := catalog.byCardID[cardID]
	if !ok {
		return contractspkg.Fixture{}, false
	}

	return cloneFixture(fixture), true
}

// Len exposes how many fixtures are currently indexed.
func (catalog FixtureCatalog) Len() int {
	return len(catalog.byCardID)
}

func (catalog FixtureCatalog) clone() FixtureCatalog {
	if len(catalog.byCardID) == 0 {
		return FixtureCatalog{byCardID: map[string]contractspkg.Fixture{}}
	}

	cloned := make(map[string]contractspkg.Fixture, len(catalog.byCardID))
	for cardID, fixture := range catalog.byCardID {
		cloned[cardID] = cloneFixture(fixture)
	}

	return FixtureCatalog{byCardID: cloned}
}

func defaultFixtureDirectory() (string, error) {
	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		return "", fmt.Errorf("runtime.Caller failed")
	}

	return filepath.Clean(filepath.Join(filepath.Dir(currentFile), "../../../shared/contracts/fixtures")), nil
}

func cloneFixture(fixture contractspkg.Fixture) contractspkg.Fixture {
	cloned := fixture
	cloned.Input.Logic.TargetKinds = slices.Clone(fixture.Input.Logic.TargetKinds)
	cloned.Input.Logic.Effects = slices.Clone(fixture.Input.Logic.Effects)
	cloned.Input.Logic.ScriptID = cloneOptionalString(fixture.Input.Logic.ScriptID)
	cloned.Expectations.TargetKinds = slices.Clone(fixture.Expectations.TargetKinds)
	cloned.Expectations.ScriptID = cloneOptionalString(fixture.Expectations.ScriptID)
	return cloned
}

func cloneOptionalString(value *string) *string {
	if value == nil {
		return nil
	}

	cloned := *value
	return &cloned
}
