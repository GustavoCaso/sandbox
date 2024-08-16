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
	WS //

	LEFT_BRACKET  // {
	RIGHT_BRACKET // }
	DOUBLE_QUOTES // "
	LETTER        // [a-zA-Z]
	COLON         // :
	COMMA         // ,
)

type token struct {
	i indentfier
	v rune
}

func (t token) String() string {
	var v string
	switch t.i {
	case LEFT_BRACKET:
		v = "{"
	case RIGHT_BRACKET:
		v = "}"
	case LETTER:
		v = string(t.v)
	case DOUBLE_QUOTES:
		v = "\""
	case COMMA:
		v = ","
	case COLON:
		v = ":"
	case WS:
		v = "**"
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
	return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || (ch >= '0' && ch <= '9')
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
		case '"':
			tokens = append(tokens, token{i: DOUBLE_QUOTES})
		case ':':
			tokens = append(tokens, token{i: COLON})
		case ',':
			tokens = append(tokens, token{i: COMMA})
		default:
			if isLetter(t) {
				tokens = append(tokens, token{i: LETTER, v: t})
				continue
			}

			if isWhitespace(t) {
				// We do not care about white space
				continue
			}

			tokens = append(tokens, token{i: ILLEGAL})
		}

	}

	fmt.Printf("%d tokens: %+v\n", len(tokens), tokens)
	if !valid(tokens) {
		os.Exit(1)
	}

	os.Exit(0)
}

func validString(tokens []token) bool {
	if !(tokens[0].i == DOUBLE_QUOTES && tokens[len(tokens)-1].i == DOUBLE_QUOTES) {
		log.Println("missing double quotes for string")
		return false
	}

	for _, token := range tokens[1 : len(tokens)-1] {
		if token.i != LETTER {
			log.Println("not valid string contents")
			return false
		}
	}

	return true
}

func valid(tokens []token) bool {
	if len(tokens) < 2 {
		return false
	}
	if !(tokens[0].i == LEFT_BRACKET && tokens[len(tokens)-1].i == RIGHT_BRACKET) {
		log.Println("missing open or close brackets")
		return false
	}
	current_key := false
	current_key_tokens := []token{}
	current_value := false
	current_value_tokens := []token{}
	pairs := map[any]any{}
	tokensToValidate := tokens[1 : len(tokens)-1]

	idx := 0
	for idx < len(tokensToValidate) {
		t := tokensToValidate[idx]
		if t.i == ILLEGAL {
			log.Println("illegal token")
			return false
		}

		if current_key && current_value {
			// We have a key value pair the next character must by a comma
			// Invallid JSON
			if t.i != COMMA {
				log.Println("missing comma")
				return false
			}

			if idx+1 == len(tokensToValidate) {
				log.Println("invalid trailing comma")
				return false
			}

			current_key = false
			current_value = false
			idx += 1
			continue
		}

		if t.i == DOUBLE_QUOTES {
			val := []token{t}
			val, advance := consumeTokenUntil(tokensToValidate[idx+1:], val, DOUBLE_QUOTES)
			if advance == 0 {
				log.Println("consumeTokenUntil invalid")
				return false
			}

			if !validString(val) {
				log.Println("invalid string")
				return false
			}

			idx += advance + 1

			if !current_key && !current_value {
				current_key = true
				current_key_tokens = val
				continue
			}

			if current_key && !current_value {
				current_value = true
				current_value_tokens = val
				stringVal := ""
				for _, t := range current_key_tokens {
					stringVal += t.String()
				}
				pairs[stringVal] = current_value_tokens
				continue
			}
		}

		if t.i == COLON {
			// Colon is used to split key value pairs
			if !current_key {
				log.Println("invalid colon")
				return false
			}

			idx += 1
		}

		if t.i == LETTER {
			log.Println("invalid key or value")
			return false
		}
	}

	fmt.Println(pairs)
	return true
}

func consumeTokenUntil(tokens []token, result []token, identifier indentfier) ([]token, int) {
	moves := 0
	for _, t := range tokens {
		moves += 1
		if t.i == identifier {
			result = append(result, t)
			break
		}

		result = append(result, t)
	}

	return result, moves
}
