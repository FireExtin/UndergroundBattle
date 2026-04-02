package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"sync"

	"undergroundbattle/server/pkg/rules"
)

// Purpose: Owns the authoritative seven-step setup state machine used before entering the playable match flow.

const (
	setupStepTotal = 7
)

var (
	errSetupNotActive    = errors.New("setup_not_active")
	errSetupNotCompleted = errors.New("setup_not_completed")
)

type SetupStartInput struct {
	Seed                uint64   `json:"seed,omitempty"`
	P1Societies         []string `json:"p1Societies,omitempty"`
	P2Societies         []string `json:"p2Societies,omitempty"`
	PreviousLoserPlayer string   `json:"previousLoserPlayer,omitempty"`
}

type SetupAdvanceInput struct {
	P1Societies            []string       `json:"p1Societies,omitempty"`
	P2Societies            []string       `json:"p2Societies,omitempty"`
	MulliganBottomCount    map[string]int `json:"mulliganBottomCount,omitempty"`
	StartingPlayerID       string         `json:"startingPlayerId,omitempty"`
	UsePreviousLoserChoice bool           `json:"usePreviousLoserChoice,omitempty"`
}

type SetupStepStatus struct {
	Step      int    `json:"step"`
	Title     string `json:"title"`
	Completed bool   `json:"completed"`
}

type SetupRegionView struct {
	CardID          string `json:"cardId"`
	DefinitionID    string `json:"definitionId"`
	Name            string `json:"name"`
	Type            string `json:"type"`
	Description     string `json:"description,omitempty"`
	FAQ             string `json:"faq,omitempty"`
	InfluenceLimit  int    `json:"influenceLimit"`
	Score           int    `json:"score"`
	RegionOrder     int    `json:"regionOrder"`
	WorldDeckIndex  int    `json:"worldDeckIndex"`
	WorldDeckRemain int    `json:"worldDeckRemain"`
}

type SetupState struct {
	Active               bool                `json:"active"`
	Completed            bool                `json:"completed"`
	CurrentStep          int                 `json:"currentStep"`
	Seed                 uint64              `json:"seed"`
	Steps                []SetupStepStatus   `json:"steps"`
	P1Societies          []string            `json:"p1Societies,omitempty"`
	P2Societies          []string            `json:"p2Societies,omitempty"`
	MarkerPoolReady      bool                `json:"markerPoolReady"`
	WorldDeckCount       int                 `json:"worldDeckCount"`
	RevealedRegions      []SetupRegionView   `json:"revealedRegions,omitempty"`
	PlayerDeckCount      map[string]int      `json:"playerDeckCount"`
	PlayerHandCount      map[string]int      `json:"playerHandCount"`
	MulliganUsed         map[string]bool     `json:"mulliganUsed"`
	StartingPlayerID     string              `json:"startingPlayerId,omitempty"`
	PreviousLoserPlayer  string              `json:"previousLoserPlayer,omitempty"`
	LastStepMessage      string              `json:"lastStepMessage,omitempty"`
	RuntimeIgnoredScopes map[string][]string `json:"runtimeIgnoredScopes,omitempty"`
	RuntimeNotes         map[string]string   `json:"runtimeNotes,omitempty"`
}

type setupRuntimeState struct {
	worldDeck       []setupCard
	revealedRegions []setupCard
	playerDeck      map[string][]setupCard
	playerHand      map[string][]setupCard
}

type setupCard struct {
	DefinitionID   string
	InstanceID     string
	Name           string
	Set            string
	CardType       string
	BasicType      string
	Society        string
	Description    string
	FAQ            string
	Cost           int
	Color          string
	Loyalty        string
	Defense        int
	InfluenceLimit int
	Score          int
	DeckCard       bool
}

type cardFileEntry struct {
	ID        string `json:"id"`
	Name      string `json:"name"`
	Set       string `json:"set"`
	Type      string `json:"type"`
	BasicType string `json:"basic-type"`
	Society   string `json:"society"`
	Color     string `json:"color"`
	Cost      string `json:"cost"`
	Loyalty   string `json:"lyl"`
	Text      string `json:"text"`
	FAQ       string `json:"FAQ"`
	DFC       string `json:"dfc"`
	Req       string `json:"req"`
	Score     string `json:"sc"`
	DeckCard  bool   `json:"deckcard"`
}

var (
	catalogOnce sync.Once
	catalogData []setupCard
	catalogErr  error
)

func (session *SandboxSession) SetupState() SetupState {
	session.mu.Lock()
	defer session.mu.Unlock()
	return cloneSetupState(session.setup)
}

func (session *SandboxSession) StartSetup(input SetupStartInput) (SetupState, error) {
	session.mu.Lock()
	defer session.mu.Unlock()

	seed := input.Seed
	if seed == 0 {
		seed = 20260402
	}

	session.setup = SetupState{
		Active:              true,
		Completed:           false,
		CurrentStep:         1,
		Seed:                seed,
		P1Societies:         slicesOrDefault(input.P1Societies, []string{"方碑序列", "帷幕守望"}),
		P2Societies:         slicesOrDefault(input.P2Societies, []string{"王座会", "国家机构"}),
		MarkerPoolReady:     false,
		WorldDeckCount:      0,
		PlayerDeckCount:     map[string]int{"P1": 0, "P2": 0},
		PlayerHandCount:     map[string]int{"P1": 0, "P2": 0},
		MulliganUsed:        map[string]bool{"P1": false, "P2": false},
		PreviousLoserPlayer: strings.TrimSpace(input.PreviousLoserPlayer),
		RuntimeIgnoredScopes: map[string][]string{
			"construct": {"society_limit", "deck_size_limit", "duplicate_limit"},
			"play":      {"queue_operation_cost_payment", "queue_operation_loyalty_requirement"},
		},
		RuntimeNotes: map[string]string{
			"pool": "当前仅使用基础包 deckcard=true 候选卡。",
			"play": "play_card 已启用费用与忠诚校验；queue_operation 仍保留调试通道兼容。",
		},
	}
	session.setup.Steps = buildSetupSteps(session.setup.CurrentStep, session.setup.Completed)
	session.setupRuntime = setupRuntimeState{
		playerDeck: map[string][]setupCard{"P1": {}, "P2": {}},
		playerHand: map[string][]setupCard{"P1": {}, "P2": {}},
	}
	session.messages = nil
	session.nextMessageNumber = 1
	session.latestReport = nil

	state := rules.NewGameState(rules.InitialStateConfig{
		GameID:         "game-sandbox-live",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           seed,
	})
	session.state = state
	session.projector = rules.NewProjectionEngine()

	return cloneSetupState(session.setup), nil
}

func (session *SandboxSession) AdvanceSetup(input SetupAdvanceInput) (SetupState, error) {
	session.mu.Lock()
	defer session.mu.Unlock()

	if !session.setup.Active {
		return SetupState{}, errSetupNotActive
	}
	if session.setup.Completed {
		return cloneSetupState(session.setup), nil
	}

	switch session.setup.CurrentStep {
	case 1:
		session.setup.P1Societies = slicesOrDefault(input.P1Societies, session.setup.P1Societies)
		session.setup.P2Societies = slicesOrDefault(input.P2Societies, session.setup.P2Societies)
		session.setup.LastStepMessage = "步骤1完成：记录双方快速组牌选择。"
		session.setup.CurrentStep = 2
	case 2:
		regions, err := loadSetupRegionDeckBaseOnly()
		if err != nil {
			return SetupState{}, err
		}
		shuffleSetupCards(regions, int64(session.setup.Seed)+11)
		session.setupRuntime.worldDeck = regions
		session.setup.WorldDeckCount = len(regions)
		session.setup.LastStepMessage = "步骤2完成：世界牌库已构建并洗牌。"
		session.setup.CurrentStep = 3
	case 3:
		session.setup.MarkerPoolReady = true
		session.setup.LastStepMessage = "步骤3完成：标志已整理。"
		session.setup.CurrentStep = 4
	case 4:
		cards, err := loadSetupPlayablePoolBaseOnly()
		if err != nil {
			return SetupState{}, err
		}
		p1Deck := cloneSetupCards(cards)
		p2Deck := cloneSetupCards(cards)
		shuffleSetupCards(p1Deck, int64(session.setup.Seed)+21)
		shuffleSetupCards(p2Deck, int64(session.setup.Seed)+22)
		attachInstances("P1", p1Deck)
		attachInstances("P2", p2Deck)
		session.setupRuntime.playerDeck["P1"] = p1Deck
		session.setupRuntime.playerDeck["P2"] = p2Deck
		session.setupRuntime.playerHand["P1"] = nil
		session.setupRuntime.playerHand["P2"] = nil
		session.setup.PlayerDeckCount["P1"] = len(p1Deck)
		session.setup.PlayerDeckCount["P2"] = len(p2Deck)
		session.setup.PlayerHandCount["P1"] = 0
		session.setup.PlayerHandCount["P2"] = 0
		session.setup.LastStepMessage = "步骤4完成：双方玩家牌库已构建并完成洗牌语义。"
		session.setup.CurrentStep = 5
	case 5:
		revealed := make([]setupCard, 0, 3)
		for len(session.setupRuntime.worldDeck) > 0 && len(revealed) < 3 {
			revealed = append(revealed, session.setupRuntime.worldDeck[0])
			session.setupRuntime.worldDeck = session.setupRuntime.worldDeck[1:]
		}
		session.setupRuntime.revealedRegions = revealed
		session.setup.WorldDeckCount = len(session.setupRuntime.worldDeck)
		session.setup.RevealedRegions = buildSetupRegionViews(revealed, len(session.setupRuntime.worldDeck))
		session.setup.LastStepMessage = "步骤5完成：翻开3张地区牌。"
		session.setup.CurrentStep = 6
	case 6:
		for _, playerID := range []string{"P1", "P2"} {
			deck := session.setupRuntime.playerDeck[playerID]
			hand := session.setupRuntime.playerHand[playerID]
			drawSetupCards(&deck, &hand, 6)
			session.setupRuntime.playerDeck[playerID] = deck
			session.setupRuntime.playerHand[playerID] = hand
		}
		applySetupMulligan(session, input)
		for _, playerID := range []string{"P1", "P2"} {
			session.setup.PlayerDeckCount[playerID] = len(session.setupRuntime.playerDeck[playerID])
			session.setup.PlayerHandCount[playerID] = len(session.setupRuntime.playerHand[playerID])
		}
		session.setup.LastStepMessage = "步骤6完成：双方抓取起始手牌并处理再调度。"
		session.setup.CurrentStep = 7
	case 7:
		startingPlayerID := resolveStartingPlayerID(session.setup.Seed, input, session.setup.PreviousLoserPlayer)
		session.setup.StartingPlayerID = startingPlayerID
		session.setup.Completed = true
		session.setup.LastStepMessage = "步骤7完成：已确定先手，进入正式对战。"
		if err := session.finalizeSetupToMatchLocked(startingPlayerID); err != nil {
			return SetupState{}, err
		}
	default:
		return SetupState{}, fmt.Errorf("setup_step_invalid")
	}

	session.setup.Steps = buildSetupSteps(session.setup.CurrentStep, session.setup.Completed)
	return cloneSetupState(session.setup), nil
}

func (session *SandboxSession) finalizeSetupToMatchLocked(startingPlayerID string) error {
	state := rules.NewGameState(rules.InitialStateConfig{
		GameID:         "game-sandbox-live",
		ActivePlayerID: startingPlayerID,
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           session.setup.Seed,
	})
	state.Board.Cards = make([]rules.CardState, 0)

	for index, region := range session.setupRuntime.revealedRegions {
		state.Board.Cards = append(state.Board.Cards, rules.CardState{
			CardID:         region.InstanceID,
			DefinitionID:   region.DefinitionID,
			Name:           region.Name,
			Description:    region.Description,
			FAQ:            region.FAQ,
			Kind:           rules.CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           rules.CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    index + 1,
			RegionScore:    region.Score,
			PrintedStats: rules.CardNumericStats{
				Influence: region.InfluenceLimit,
			},
			EffectiveStats: rules.CardNumericStats{
				Influence: region.InfluenceLimit,
			},
		})
	}

	for _, playerID := range []string{"P1", "P2"} {
		for _, card := range session.setupRuntime.playerDeck[playerID] {
			state.Board.Cards = append(state.Board.Cards, setupCardToState(card, playerID, rules.CardZoneDeck, false))
		}
		for _, card := range session.setupRuntime.playerHand[playerID] {
			state.Board.Cards = append(state.Board.Cards, setupCardToState(card, playerID, rules.CardZoneHand, false))
		}
	}

	state.Turn.ActivePlayerID = startingPlayerID
	state.Turn.PriorityPlayerID = startingPlayerID
	state.Turn.Priority.CurrentPlayerID = startingPlayerID
	state.Turn.Priority.LastPassedPlayerID = ""
	state.Turn.Priority.PassCount = 0
	state.Turn.FirstPlayerPrivilegeUsed = false

	views := session.projector.Generate(state)
	messages, err := session.materializeBootstrapMessages(views)
	if err != nil {
		return err
	}

	session.state = state
	session.messages = cloneProtocolEnvelopes(messages)
	session.nextMessageNumber = len(messages) + 1
	return nil
}

func setupCardToState(card setupCard, ownerID string, zone rules.CardZone, revealed bool) rules.CardState {
	kind := rules.CardKindUnknown
	basic := strings.TrimSpace(card.BasicType)
	switch {
	case strings.Contains(basic, "角色"):
		kind = rules.CardKindCharacter
	case strings.Contains(basic, "附属"):
		kind = rules.CardKindAsset
	case strings.Contains(basic, "事务"):
		kind = rules.CardKindEvent
	}

	state := rules.CardState{
		CardID:         card.InstanceID,
		DefinitionID:   card.DefinitionID,
		Name:           card.Name,
		Description:    card.Description,
		FAQ:            card.FAQ,
		Cost:           card.Cost,
		Color:          card.Color,
		Loyalty:        card.Loyalty,
		Kind:           kind,
		OwnerID:        ownerID,
		Zone:           zone,
		VisibleToOwner: true,
		Revealed:       revealed,
	}

	if kind == rules.CardKindCharacter {
		defense := card.Defense
		if defense <= 0 {
			defense = 1
		}
		baseStats := rules.CardNumericStats{
			Combat:        1,
			Defense:       defense,
			Investigation: 1,
		}
		state.PrintedStats = baseStats
		state.EffectiveStats = baseStats
	}

	return state
}

func applySetupMulligan(session *SandboxSession, input SetupAdvanceInput) {
	if len(input.MulliganBottomCount) == 0 {
		return
	}

	for _, playerID := range []string{"P1", "P2"} {
		bottomCount := input.MulliganBottomCount[playerID]
		if bottomCount <= 0 {
			continue
		}
		if session.setup.MulliganUsed[playerID] {
			continue
		}
		hand := session.setupRuntime.playerHand[playerID]
		if bottomCount > len(hand) {
			bottomCount = len(hand)
		}
		if bottomCount == 0 {
			continue
		}

		mulliganPart := cloneSetupCards(hand[:bottomCount])
		session.setupRuntime.playerHand[playerID] = cloneSetupCards(hand[bottomCount:])
		session.setupRuntime.playerDeck[playerID] = append(session.setupRuntime.playerDeck[playerID], mulliganPart...)
		shuffleSetupCards(session.setupRuntime.playerDeck[playerID], int64(session.setup.Seed)+int64(bottomCount)+31)
		deck := session.setupRuntime.playerDeck[playerID]
		nextHand := session.setupRuntime.playerHand[playerID]
		drawSetupCards(&deck, &nextHand, bottomCount)
		session.setupRuntime.playerDeck[playerID] = deck
		session.setupRuntime.playerHand[playerID] = nextHand
		session.setup.MulliganUsed[playerID] = true
	}
}

func resolveStartingPlayerID(seed uint64, input SetupAdvanceInput, previousLoser string) string {
	if input.UsePreviousLoserChoice && strings.TrimSpace(previousLoser) != "" {
		candidate := strings.TrimSpace(input.StartingPlayerID)
		if candidate == "P1" || candidate == "P2" {
			return candidate
		}
	}

	if input.StartingPlayerID == "P1" || input.StartingPlayerID == "P2" {
		return input.StartingPlayerID
	}

	rng := rand.New(rand.NewSource(int64(seed) + 41))
	if rng.Intn(2) == 0 {
		return "P1"
	}
	return "P2"
}

func drawSetupCards(deck *[]setupCard, hand *[]setupCard, count int) {
	if deck == nil || hand == nil || count <= 0 {
		return
	}
	for i := 0; i < count && len(*deck) > 0; i++ {
		*hand = append(*hand, (*deck)[0])
		*deck = (*deck)[1:]
	}
}

func buildSetupSteps(currentStep int, setupCompleted bool) []SetupStepStatus {
	titles := []string{
		"玩家选择牌组",
		"设置世界牌库",
		"整理标志",
		"设置玩家牌库",
		"翻开地区牌",
		"抓取起始手牌",
		"确定先手玩家",
	}
	result := make([]SetupStepStatus, 0, len(titles))
	for index, title := range titles {
		step := index + 1
		completed := step < currentStep || (setupCompleted && step == currentStep)
		result = append(result, SetupStepStatus{Step: step, Title: title, Completed: completed})
	}
	return result
}

func buildSetupRegionViews(cards []setupCard, worldDeckRemain int) []SetupRegionView {
	result := make([]SetupRegionView, 0, len(cards))
	for index, card := range cards {
		result = append(result, SetupRegionView{
			CardID:          card.InstanceID,
			DefinitionID:    card.DefinitionID,
			Name:            card.Name,
			Type:            card.CardType,
			Description:     card.Description,
			FAQ:             card.FAQ,
			InfluenceLimit:  card.InfluenceLimit,
			Score:           card.Score,
			RegionOrder:     index + 1,
			WorldDeckIndex:  index,
			WorldDeckRemain: worldDeckRemain,
		})
	}
	return result
}

func cloneSetupState(state SetupState) SetupState {
	cloned := state
	cloned.P1Societies = append([]string(nil), state.P1Societies...)
	cloned.P2Societies = append([]string(nil), state.P2Societies...)
	cloned.Steps = append([]SetupStepStatus(nil), state.Steps...)
	cloned.RevealedRegions = append([]SetupRegionView(nil), state.RevealedRegions...)
	cloned.PlayerDeckCount = cloneStringIntMap(state.PlayerDeckCount)
	cloned.PlayerHandCount = cloneStringIntMap(state.PlayerHandCount)
	cloned.MulliganUsed = cloneStringBoolMap(state.MulliganUsed)
	cloned.RuntimeIgnoredScopes = cloneStringSliceMap(state.RuntimeIgnoredScopes)
	cloned.RuntimeNotes = cloneStringStringMap(state.RuntimeNotes)
	return cloned
}

func cloneStringIntMap(source map[string]int) map[string]int {
	if len(source) == 0 {
		return map[string]int{}
	}
	result := make(map[string]int, len(source))
	for key, value := range source {
		result[key] = value
	}
	return result
}

func cloneStringBoolMap(source map[string]bool) map[string]bool {
	if len(source) == 0 {
		return map[string]bool{}
	}
	result := make(map[string]bool, len(source))
	for key, value := range source {
		result[key] = value
	}
	return result
}

func cloneStringSliceMap(source map[string][]string) map[string][]string {
	if len(source) == 0 {
		return map[string][]string{}
	}
	result := make(map[string][]string, len(source))
	for key, values := range source {
		result[key] = append([]string(nil), values...)
	}
	return result
}

func cloneStringStringMap(source map[string]string) map[string]string {
	if len(source) == 0 {
		return map[string]string{}
	}
	result := make(map[string]string, len(source))
	for key, value := range source {
		result[key] = value
	}
	return result
}

func slicesOrDefault(values []string, fallback []string) []string {
	if len(values) == 0 {
		return append([]string(nil), fallback...)
	}
	result := make([]string, 0, len(values))
	for _, value := range values {
		trimmed := strings.TrimSpace(value)
		if trimmed == "" {
			continue
		}
		result = append(result, trimmed)
	}
	if len(result) == 0 {
		return append([]string(nil), fallback...)
	}
	return result
}

func cloneSetupCards(cards []setupCard) []setupCard {
	return append([]setupCard(nil), cards...)
}

func attachInstances(ownerID string, cards []setupCard) {
	for index := range cards {
		cards[index].InstanceID = fmt.Sprintf("%s-%s-%03d", ownerID, cards[index].DefinitionID, index+1)
	}
}

func shuffleSetupCards(cards []setupCard, seed int64) {
	if len(cards) <= 1 {
		return
	}
	rng := rand.New(rand.NewSource(seed))
	rng.Shuffle(len(cards), func(i int, j int) {
		cards[i], cards[j] = cards[j], cards[i]
	})
}

func loadSetupRegionDeckBaseOnly() ([]setupCard, error) {
	cards, err := loadSetupCatalogCards()
	if err != nil {
		return nil, err
	}
	allowed := map[string]bool{
		"DQJC107": true,
		"DQJC108": true,
		"DQJC109": true,
		"DQJC110": true,
		"DQJC111": true,
		"DQJC112": true,
		"DQJC113": true,
		"DQJC114": true,
		"DQJC115": true,
		"DQJC116": true,
	}
	regions := make([]setupCard, 0, 10)
	for _, card := range cards {
		if card.Set != "基础" {
			continue
		}
		if !allowed[card.DefinitionID] {
			continue
		}
		copyCard := card
		copyCard.InstanceID = card.DefinitionID
		regions = append(regions, copyCard)
	}
	sort.Slice(regions, func(i int, j int) bool {
		return regions[i].DefinitionID < regions[j].DefinitionID
	})
	return regions, nil
}

func loadSetupPlayablePoolBaseOnly() ([]setupCard, error) {
	cards, err := loadSetupCatalogCards()
	if err != nil {
		return nil, err
	}
	playable := make([]setupCard, 0, 128)
	for _, card := range cards {
		if card.Set != "基础" || !card.DeckCard {
			continue
		}
		if strings.TrimSpace(card.BasicType) == "地区" {
			continue
		}
		playable = append(playable, card)
	}
	sort.Slice(playable, func(i int, j int) bool {
		return playable[i].DefinitionID < playable[j].DefinitionID
	})
	return playable, nil
}

func loadSetupCatalogCards() ([]setupCard, error) {
	catalogOnce.Do(func() {
		root, err := repositoryRootFromSetupFile()
		if err != nil {
			catalogErr = err
			return
		}
		cardsRoot := filepath.Join(root, "organized_content", "cards")
		entries, err := os.ReadDir(cardsRoot)
		if err != nil {
			catalogErr = err
			return
		}

		loaded := make([]setupCard, 0, 1024)
		for _, entry := range entries {
			if !entry.IsDir() {
				continue
			}
			filePath := filepath.Join(cardsRoot, entry.Name(), "cards.json")
			data, err := os.ReadFile(filePath)
			if err != nil {
				continue
			}
			var payload map[string]cardFileEntry
			if err := json.Unmarshal(data, &payload); err != nil {
				continue
			}
			for definitionID, raw := range payload {
				id := strings.TrimSpace(raw.ID)
				if id == "" {
					id = strings.TrimSpace(definitionID)
				}
				if id == "" {
					continue
				}
				loaded = append(loaded, setupCard{
					DefinitionID:   id,
					Name:           strings.TrimSpace(raw.Name),
					Set:            strings.TrimSpace(raw.Set),
					CardType:       strings.TrimSpace(raw.Type),
					BasicType:      strings.TrimSpace(raw.BasicType),
					Society:        strings.TrimSpace(raw.Society),
					Cost:           parseCardCost(raw.Cost),
					Color:          strings.TrimSpace(raw.Color),
					Loyalty:        strings.TrimSpace(raw.Loyalty),
					Description:    strings.TrimSpace(raw.Text),
					FAQ:            strings.TrimSpace(raw.FAQ),
					Defense:        parseCardInt(raw.DFC),
					InfluenceLimit: parseCardInt(raw.Req),
					Score:          parseCardInt(raw.Score),
					DeckCard:       raw.DeckCard,
				})
			}
		}
		sort.Slice(loaded, func(i int, j int) bool {
			return loaded[i].DefinitionID < loaded[j].DefinitionID
		})
		catalogData = loaded
	})

	if catalogErr != nil {
		return nil, catalogErr
	}
	return append([]setupCard(nil), catalogData...), nil
}

func parseCardInt(raw string) int {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		return 0
	}
	parsed, err := strconv.Atoi(trimmed)
	if err != nil {
		return 0
	}
	return parsed
}

func parseCardCost(raw string) int {
	return parseCardInt(raw)
}

func repositoryRootFromSetupFile() (string, error) {
	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		return "", fmt.Errorf("runtime.Caller failed")
	}
	return filepath.Clean(filepath.Join(filepath.Dir(currentFile), "../../..")), nil
}
