package main

import (
	"bufio"
	"errors"
	"fmt"
	"io"
	"log"
	"os"
)

type indentfier int

const (
	// Special tokens
	ILLEGAL indentfier = iota
	EOF

	LEFT_BRACKET  // {
	RIGHT_BRACKET // }
)

type token struct {
	i indentfier
}

func (t token) String() string {
	var v string
	switch t.i {
	case LEFT_BRACKET:
		v = "{"
	case RIGHT_BRACKET:
		v = "}"
	}

	return v
}

type scanner struct {
	r *bufio.Reader
}

var eof = rune(0)

func newScanner(r io.Reader) *scanner {
	return &scanner{
		r: bufio.NewReader(r),
	}
}

func (s *scanner) Read() rune {
	r, _, err := s.r.ReadRune()

	if err != nil {
		return eof
	}
	return r
}

func (s *scanner) Unread() { _ = s.r.UnreadRune() }

func isWhitespace(ch rune) bool {
	return ch == ' ' || ch == '\t' || ch == '\n'
}

func isLetter(ch rune) bool {
	return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z')
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("usage `go run main.go <testfile>`")
		os.Exit(1)
	}
	testFile := os.Args[1]

	_, err := os.Stat(testFile)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			fmt.Printf("file %s does not exists\n", testFile)
			os.Exit(1)
		}
		log.Fatal(err)
	}

	reader, err := os.Open(testFile)
	if err != nil {
		log.Fatal(err)
	}

	tokens := []token{}
	scanner := newScanner(reader)

	for {
		t := scanner.Read()

		if t == eof {
			break
		}

		switch t {
		case '{':
			tokens = append(tokens, token{i: LEFT_BRACKET})
		case '}':
			tokens = append(tokens, token{i: RIGHT_BRACKET})
		}

	}

	fmt.Printf("%d tokens: %+v\n", len(tokens), tokens)
	if !valid(tokens) {
		os.Exit(1)
	}

	os.Exit(0)
}

func valid(tokens []token) bool {
	if len(tokens) < 2 {
		return false
	}
	return tokens[0].i == LEFT_BRACKET && tokens[len(tokens)-1].i == RIGHT_BRACKET
}
