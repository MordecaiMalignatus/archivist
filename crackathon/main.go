package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"
	// "github.com/charmbracelet/huh"
)

const (
	archivePath     string = "~/grimoire/mtg-archive.json"
	scryfallApiRoot string = "https://api.scryfall.com/"
)

func main() {
	if len(os.Args[1:]) != 1 {
		log.Fatalf("Need exactly 1 positional argument, the setcode, found %d\n", len(os.Args))
	}
	setCode := strings.ToUpper(os.Args[1])
	reader := bufio.NewReader(os.Stdin)

	for {
		fmt.Printf("Enter card number for set %s: \n", setCode)
		line, err := reader.ReadString('\n')
		if err != nil {
			log.Fatalf("couldn't read from stdin: %v\n", err)
		}
		line = strings.TrimSpace(line)
		parsedLine, err := strconv.Atoi(line)
		if err != nil {
			log.Printf("[ERR] Could not parse '%s' into a number, skipping...\n", line)
			continue
		}

		card, err := getCard(setCode, parsedLine)
		if err != nil {
			log.Printf("[ERR] Could not obtain information from Scryfall, skipping this input: %v\n", err)
			continue
		}
		_ = addToArchive(card)

		log.Printf("Added %s to collection!\n", card.Name)
	}
}

type Card struct {
	OracleId string `json:"oracle_id,omitempty"`
	Uri      string `json:"uri,omitempty"`
	Name     string `json:"name"`
	Set      string `json:"set"`
	SetName  string `json:"set_name,omitempty"`
	Rarity   string `json:"rarity,omitempty"`
	Count    int    `json:"count"`
}

type Archive map[string][]Card

func addToArchive(c Card) *Archive {
	f, err := os.Open(archivePath)
	if err != nil {
		log.Fatalf("Can't open archive at %s for reading: %v\n", archivePath, err)
	}

	fileContent, err := io.ReadAll(f)
	if err != nil {
		log.Fatalf("Can't read from archive file: %v", err)
	}
	f.Close()

	var a Archive
	err = json.Unmarshal(fileContent, &a)
	if err != nil {
		log.Fatalf("Archive is not valid JSON: %v\n", err)
	}

	existingCards, hasSet := a[c.Set]
	if !hasSet {
		a[c.Set] = []Card{c}
	} else {
		existingCards = append(existingCards, c)
		a[c.Set] = existingCards
	}

	futureFileContent, err := json.Marshal(a)
	if err != nil {
		log.Fatalf("Can't marshal archive contents, JSON encoder error: %v", err)
	}
	f, err = os.Create(archivePath)
	if err != nil {
		log.Fatalf("Can't open archive for writing back after editing: %v", err)
	}
	f.Write(futureFileContent)

	return &a
}

func getCard(setCode string, cardNumber int) (Card, error) {
	url := fmt.Sprintf("%s/cards/%s/%d", scryfallApiRoot, setCode, cardNumber)
	res, err := http.Get(url)
	if err != nil {
		return Card{}, fmt.Errorf("can't retrieve card from scryfall: %w", err)
	}
	body, _ := io.ReadAll(res.Body)

	var cardResult Card
	err = json.Unmarshal(body, &cardResult)
	if err != nil {
		return Card{}, fmt.Errorf("can't deserialize json: %w", err)
	}

	return cardResult, nil
}
