package main

import (
	"bufio"
	"bytes"
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

	LEFT_CURLY_BRACKET  // {
	RIGHT_CURLY_BRACKET // }
	LEFT_BRACKET        // [
	RIGHT_BRACKET       // ]
	DOUBLE_QUOTES       // "
	LETTER              // [a-zA-Z]
	COLON               // :
	COMMA               // ,
	NUMBER              // [0-9]
	BOOLEAN             // (true|false)
	NULL                // null

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

type parser struct {
	s                 *scanner
	firstCurlyBracket bool
}

func (p *parser) Parse() []token {
	tokens := []token{}

	for {
		t := p.s.Read()

		if t == eof {
			break
		}

		switch t {
		case '{':
			if !p.firstCurlyBracket {
				tokens = append(tokens, token{i: LEFT_CURLY_BRACKET, val: "{"})
				p.firstCurlyBracket = true
				continue
			}

			values, err := p.s.ReadUntilNext('}')

			if err {
				log.Fatal("failed to parse object")
			}

			values = values[:len(values)-1]

			buf := bytes.Buffer{}
			for _, val := range values {
				buf.WriteRune(val)
			}

			objectScanner := newScanner(bytes.NewReader(buf.Bytes()))
			objectParser := &parser{
				s:                 objectScanner,
				firstCurlyBracket: true,
			}

			objectTokens := objectParser.Parse()

			sbuf := bytes.Buffer{}
			sbuf.WriteString("{")
			for _, val := range objectTokens {
				sbuf.WriteString(val.String())
			}
			sbuf.WriteString("}")

			tokens = append(tokens, token{i: OBJECT, val: sbuf.String()})
		case '}':
			if !p.firstCurlyBracket {
				log.Fatal("incorrect parser state")
			}
			tokens = append(tokens, token{i: RIGHT_CURLY_BRACKET, val: "}"})
		case '"':
			values, err := p.s.ReadUntilNext('"')

			if err {
				log.Fatal("failed to parse string")
			}

			values = append([]rune{t}, values...)

			tokens = append(tokens, token{i: KEY_OR_VALUE, val: string(values)})
		case ':':
			tokens = append(tokens, token{i: COLON, val: ":"})
		case ',':
			tokens = append(tokens, token{i: COMMA, val: ","})
		case '[':
			values, err := p.s.ReadUntilNext(']')

			if err {
				log.Fatal("failed to parse array")
			}

			values = values[:len(values)-1]

			buf := bytes.Buffer{}
			for _, val := range values {
				buf.WriteRune(val)
			}

			arrayScanner := newScanner(bytes.NewReader(buf.Bytes()))
			arrayParser := &parser{
				s:                 arrayScanner,
				firstCurlyBracket: true,
			}

			arrayTokens := arrayParser.Parse()

			sbuf := bytes.Buffer{}
			sbuf.WriteString("[")
			for _, val := range arrayTokens {
				sbuf.WriteString(val.String())
			}
			sbuf.WriteString("]")

			tokens = append(tokens, token{i: ARRAY, val: sbuf.String()})
		default:
			if t == '-' || isDigit(t) {
				value, err := p.parseNumber(t)
				if err != nil {
					log.Fatal(err)
				}

				tokens = append(tokens, token{i: NUMBER, val: string(value)})
				continue
			}

			if isLetter(t) {
				if t == 't' {
					values, err := p.s.ReadNext(3)
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
					values, err := p.s.ReadNext(4)
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
					values, err := p.s.ReadNext(3)
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

			log.Fatalf("invalid character: %s\n", string(t))
		}
	}

	return tokens
}

func (p *parser) parseNumber(initRune rune) ([]rune, error) {
	mustIncludeFractionAtStart := false
	fractionIncluded := false
	negativeNumber := false
	exponentIncluded := false
	exponentSignIncluded := false
	start := true

	if initRune == '0' {
		mustIncludeFractionAtStart = true
	}

	if initRune == '-' {
		negativeNumber = true
	}

	value := []rune{initRune}
	for {
		v := p.s.Read()
		// multiple fraction
		if v == '.' && fractionIncluded {
			return []rune{}, errors.New("invalid number. multiple fractions")
		}

		// negative number with just fraction
		if v == '.' && negativeNumber && start {
			return []rune{}, errors.New("invalid negative number")
		}

		// leading zero without fraction
		if mustIncludeFractionAtStart && v != '.' && start {
			return []rune{}, errors.New("number start with zero, must be followed by fraction")
		}

		// leaduing fraction
		if !mustIncludeFractionAtStart && v == '.' && start {
			return []rune{}, errors.New("invalid fraction numbre")
		}

		if mustIncludeFractionAtStart && v == '.' && !start {
			start = false
			fractionIncluded = true
			value = append(value, v)
			continue
		}

		if isDigit(v) || v == '.' {
			start = false
			if v == '.' {
				fractionIncluded = true
			}
			value = append(value, v)
			continue
		}

		if v == 'E' || v == 'e' {
			if exponentIncluded {
				return []rune{}, errors.New("invalid number. multiple exponents")
			}

			exponentIncluded = true
			value = append(value, v)
			continue
		}

		if v == '+' || v == '-' {
			if exponentIncluded {
				if exponentSignIncluded {
					return []rune{}, errors.New("invalid number. multiple exponents signs")
				}

				exponentSignIncluded = true
				value = append(value, v)
				continue
			}
			return []rune{}, errors.New("invalid number. invalid sign")
		}

		// We no longer are a digit
		p.s.Unread()
		return value, nil
	}
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

	scanner := newScanner(reader)
	parser := &parser{
		s: scanner,
	}

	tokens := parser.Parse()

	fmt.Printf("%d tokens: %+v\n", len(tokens), tokens)
	if !valid(tokens) {
		os.Exit(1)
	}
	fmt.Println("valid!")
	os.Exit(0)
}

func valid(tokens []token) bool {
	if len(tokens) < 2 {
		return false
	}
	if !(tokens[0].i == LEFT_CURLY_BRACKET && tokens[len(tokens)-1].i == RIGHT_CURLY_BRACKET) {
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

		if t.i == BOOLEAN || t.i == NULL || t.i == NUMBER || t.i == ARRAY || t.i == OBJECT {
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
