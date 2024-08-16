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
	WS // whitespace

	LEFT_BRACKET  // {
	RIGHT_BRACKET // }
	DOUBLE_QUOTES // "
	LETTER        // [a-zA-Z]
	COLON         // :
	COMMA         // ,
	NUMBER        // [0-9]
	BOOLEAN       // (true|false)
	NULL          // null

	KEY_OR_VALUE // "(LETTER|NUMBER)"

	// Complex
	ARRAY
	OBJECT
)

type token struct {
	i   indentfier
	val string
}

func (t token) String() string {
	return t.val
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

func (s *scanner) ReadUntilNext(i rune) ([]rune, bool) {
	result := []rune{}

	for {
		ch := s.Read()

		if ch == eof {
			return result, true
		}

		result = append(result, ch)
		if ch == i {
			break
		}
	}

	return result, false
}

func (s *scanner) ReadNext(next int) ([]rune, bool) {
	result := []rune{}

	for i := 0; i < next; i++ {
		ch := s.Read()

		if ch == eof {
			return result, true
		}

		result = append(result, ch)
	}

	return result, false
}

func isWhitespace(ch rune) bool {
	return ch == ' ' || ch == '\t' || ch == '\n'
}

func isLetter(ch rune) bool {
	return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z')
}

func isDigit(ch rune) bool {
	return (ch >= '0' && ch <= '9')
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
			tokens = append(tokens, token{i: LEFT_BRACKET, val: "{"})
		case '}':
			tokens = append(tokens, token{i: RIGHT_BRACKET, val: "}"})
		case '"':
			values, err := scanner.ReadUntilNext('"')

			if err {
				log.Fatal("failed to parse string")
			}

			values = append([]rune{t}, values...)

			tokens = append(tokens, token{i: KEY_OR_VALUE, val: string(values)})
		case ':':
			tokens = append(tokens, token{i: COLON, val: ":"})
		case ',':
			tokens = append(tokens, token{i: COMMA, val: ","})
		default:
			if isDigit(t) {
				value := []rune{t}
				for {
					v := scanner.Read()
					if isDigit(v) {
						value = append(value, v)
					} else {
						// We no longer are a digit
						scanner.Unread()
						break
					}
				}
				tokens = append(tokens, token{i: NUMBER, val: string(value)})
				continue
			}

			if isLetter(t) {
				if t == 't' {
					values, err := scanner.ReadNext(3)
					if err {
						log.Fatal("failed to parse true value")
					}
					result := append([]rune{t}, values...)

					if string(result) != "true" {
						log.Fatal("invalid true token")
					}
					tokens = append(tokens, token{i: BOOLEAN, val: string(result)})
					continue
				}

				if t == 'f' {
					values, err := scanner.ReadNext(4)
					if err {
						log.Fatal("failed to parse false value")
					}
					result := append([]rune{t}, values...)
					if string(result) != "false" {
						log.Fatal("invalid false token")
					}
					tokens = append(tokens, token{i: BOOLEAN, val: string(result)})
					continue
				}

				if t == 'n' {
					values, err := scanner.ReadNext(3)
					if err {
						log.Fatal("failed to parse null value")
					}
					result := append([]rune{t}, values...)
					if string(result) != "null" {
						log.Fatal("invalid null token")
					}
					tokens = append(tokens, token{i: NULL, val: string(result)})
					continue
				}

				log.Fatalf("invalid letter token: %s\n", string(t))
			}

			if isWhitespace(t) {
				// We do not care about white space
				continue
			}

			tokens = append(tokens, token{i: ILLEGAL, val: "**"})
		}

	}

	fmt.Printf("%d tokens: %+v\n", len(tokens), tokens)
	if !valid(tokens) {
		os.Exit(1)
	}
	fmt.Println("valid!")
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
	tokensToValidate := tokens[1 : len(tokens)-1]

	idx := 0
	for idx < len(tokensToValidate) {
		t := tokensToValidate[idx]
		if t.i == ILLEGAL {
			log.Println("illegal token")
			return false
		}

		if t.i == KEY_OR_VALUE {
			if !current_key {
				advance := consumeTokenUntil(tokensToValidate[idx+1:], COLON)
				if advance == 0 {
					log.Println("missing colon after key")
					return false
				}
				current_key = true
				idx += advance + 1
				continue
			}

			advance := consumeTokenUntil(tokensToValidate[idx+1:], COMMA)
			if advance == 0 {
				if !(idx+1 == len(tokensToValidate)) {
					log.Println("missing comma")
					return false
				}
				idx += 1
				continue
			}

			if idx+advance+1 == len(tokensToValidate) {
				log.Println("trailing comma")
				return false
			}

			current_key = false
			idx += advance + 1
			continue
		}

		if t.i == BOOLEAN || t.i == NULL || t.i == NUMBER {
			if !current_key {
				log.Println("invalid value without a key")
				return false
			}

			advance := consumeTokenUntil(tokensToValidate[idx+1:], COMMA)
			if advance == 0 {
				if !(idx+1 == len(tokensToValidate)) {
					log.Println("missing comma")
					return false
				}
				idx += 1
				continue
			}
			if idx+advance+1 == len(tokensToValidate) {
				log.Println("trailing comma")
				return false
			}
			current_key = false
			idx += advance + 1
			continue
		}
	}

	return true
}

func consumeTokenUntil(tokens []token, identifier indentfier) int {
	moves := 0
	for _, t := range tokens {
		moves += 1
		if t.i == identifier {
			break
		}
	}

	return moves
}
