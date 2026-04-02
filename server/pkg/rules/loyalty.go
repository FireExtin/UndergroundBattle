package rules

import (
	"sort"
	"strings"
)

// Purpose: Centralizes loyalty color parsing and normalization so rules and clients can share one vocabulary.

type loyaltyColorMapping struct {
	Canonical string
	Aliases   []string
}

type loyaltyColorToken struct {
	Token      string
	Canonical  string
	RuneLength int
}

var loyaltyColorMappings = []loyaltyColorMapping{
	{Canonical: "黄色", Aliases: []string{"黄"}},
	{Canonical: "红色", Aliases: []string{"红"}},
	{Canonical: "绿色", Aliases: []string{"绿"}},
	{Canonical: "蓝色", Aliases: []string{"蓝"}},
	{Canonical: "黑色", Aliases: []string{"黑"}},
	{Canonical: "白色", Aliases: []string{"白"}},
	{Canonical: "紫色", Aliases: []string{"紫"}},
	{Canonical: "灰色", Aliases: []string{"灰"}},
}

var loyaltyColorTokens = buildLoyaltyColorTokens()

func loyaltyColorAliases() []LoyaltyColorAlias {
	aliases := make([]LoyaltyColorAlias, 0, len(loyaltyColorMappings))
	for _, mapping := range loyaltyColorMappings {
		aliases = append(aliases, LoyaltyColorAlias{
			Canonical: mapping.Canonical,
			Aliases:   append([]string(nil), mapping.Aliases...),
		})
	}
	return aliases
}

func parseLoyaltyRequirements(raw string) map[string]int {
	text := strings.TrimSpace(raw)
	if text == "" || text == "-" {
		return map[string]int{}
	}

	requirements := make(map[string]int)
	runes := []rune(text)
	for cursor := 0; cursor < len(runes); {
		matched := false
		for _, token := range loyaltyColorTokens {
			if cursor+token.RuneLength > len(runes) {
				continue
			}
			if string(runes[cursor:cursor+token.RuneLength]) != token.Token {
				continue
			}
			requirements[token.Canonical]++
			cursor += token.RuneLength
			matched = true
			break
		}
		if matched {
			continue
		}
		cursor++
	}
	return requirements
}

func buildLoyaltyColorTokens() []loyaltyColorToken {
	tokens := make([]loyaltyColorToken, 0, len(loyaltyColorMappings)*2)
	seen := make(map[string]struct{})
	add := func(token string, canonical string) {
		normalized := strings.TrimSpace(token)
		if normalized == "" {
			return
		}
		key := canonical + ":" + normalized
		if _, exists := seen[key]; exists {
			return
		}
		seen[key] = struct{}{}
		tokens = append(tokens, loyaltyColorToken{
			Token:      normalized,
			Canonical:  canonical,
			RuneLength: len([]rune(normalized)),
		})
	}

	for _, mapping := range loyaltyColorMappings {
		add(mapping.Canonical, mapping.Canonical)
		for _, alias := range mapping.Aliases {
			add(alias, mapping.Canonical)
		}
	}

	sort.Slice(tokens, func(i int, j int) bool {
		if tokens[i].RuneLength == tokens[j].RuneLength {
			return tokens[i].Token < tokens[j].Token
		}
		return tokens[i].RuneLength > tokens[j].RuneLength
	})
	return tokens
}

func normalizeLoyaltyColor(raw string) string {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" || trimmed == "中立" {
		return ""
	}
	for _, mapping := range loyaltyColorMappings {
		if trimmed == mapping.Canonical {
			return mapping.Canonical
		}
		for _, alias := range mapping.Aliases {
			if trimmed == alias {
				return mapping.Canonical
			}
		}
	}
	return ""
}
