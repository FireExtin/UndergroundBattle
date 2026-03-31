package contracts

import (
	"encoding/json"
	"os"

	contractspkg "undergroundbattle/server/pkg/contracts"
)

// LoadFixture reads a shared contract fixture so Go and TypeScript can use the same inputs.
func LoadFixture(path string) (contractspkg.Fixture, error) {
	return decodeJSON[contractspkg.Fixture](path)
}

func decodeJSON[T any](path string) (T, error) {
	var value T

	data, err := os.ReadFile(path)
	if err != nil {
		return value, err
	}

	if err := json.Unmarshal(data, &value); err != nil {
		return value, err
	}

	return value, nil
}
