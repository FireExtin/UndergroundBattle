package contracts

// BasicEffect is the minimal effect surface shared by the CardLogic DSL fixtures.
type BasicEffect struct {
	Kind      string `json:"kind"`
	TargetRef string `json:"targetRef"`
	Amount    *int   `json:"amount,omitempty"`
	Stat      string `json:"stat,omitempty"`
	Keyword   string `json:"keyword,omitempty"`
}

// FixtureCard stores the human-readable card identity and organized-content source used by a fixture.
type FixtureCard struct {
	Name       string `json:"name"`
	SourcePath string `json:"sourcePath"`
	BasicType  string `json:"basicType"`
}

// CardLogic is authored in TypeScript but interpreted by Go as the final semantic authority.
type CardLogic struct {
	ID            string        `json:"id"`
	SchemaVersion string        `json:"schemaVersion"`
	Speed         string        `json:"speed"`
	TargetKinds   []string      `json:"targetKinds"`
	RequiresStack bool          `json:"requiresStack"`
	DurationKind  string        `json:"durationKind"`
	ScriptID      *string       `json:"scriptId"`
	Effects       []BasicEffect `json:"effects"`
}

// FixtureInput wraps the authored CardLogic payload shared by both runtimes.
type FixtureInput struct {
	Logic CardLogic `json:"logic"`
}

// FixtureExpectations declares the contract assertions both runtimes must satisfy.
type FixtureExpectations struct {
	ParseOK       bool     `json:"parseOk"`
	Speed         string   `json:"speed"`
	TargetKinds   []string `json:"targetKinds"`
	RequiresStack bool     `json:"requiresStack"`
	DurationKind  string   `json:"durationKind"`
	ScriptID      *string  `json:"scriptId"`
}

// Fixture defines a self-contained CardLogic contract fixture that gates cards into the main pool.
type Fixture struct {
	CardID        string              `json:"cardId"`
	SchemaVersion string              `json:"schemaVersion"`
	Card          FixtureCard         `json:"card"`
	Input         FixtureInput        `json:"input"`
	Expectations  FixtureExpectations `json:"expectations"`
}

// ParsedCardLogic is the Go-side interpretation of the authored fixture payload.
type ParsedCardLogic struct {
	CardID            string
	CardName          string
	SourcePath        string
	BasicType         string
	LogicID           string
	SchemaVersion     string
	Speed             string
	TargetKinds       []string
	RequiresStack     bool
	DurationKind      string
	ScriptID          *string
	RequiresScript    bool
	PureDSLExecutable bool
	Effects           []BasicEffect
	EffectKinds       []string
}
