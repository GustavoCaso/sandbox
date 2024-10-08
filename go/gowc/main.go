package main

import (
	"bufio"
	"bytes"
	"flag"
	"fmt"
	"io"
	"os"
	"strings"
)

var bytesFlag = flag.Bool("c", false, "The number of bytes in each input file is written to the standard output.")
var wordsFlag = flag.Bool("w", false, "The number of words in each input file is written to the standard output.")
var linesFlag = flag.Bool("l", false, "The number of lines in each input file is written to the standard output.")
var characterFlag = flag.Bool("m", false, "The number of character in each input file is written to the standard output.")

func main() {
	flag.Parse()

	argsLength := flag.NArg()

	var reader io.Reader
	var err error
	var fileName string
	var resetFunction func(io.Reader) io.Reader

	if argsLength != 1 {
		stdin := bufio.NewReader(os.Stdin)
		dupBuf := new(bytes.Buffer)
		reader = io.TeeReader(stdin, dupBuf)

		resetFunction = func(r io.Reader) io.Reader {
			return dupBuf
		}
	} else {
		fileName := flag.Arg(0)
		reader, err = os.Open(fileName)

		defer func() {
			fileReader := reader.(*os.File)
			err := fileReader.Close()
			if err != nil {
				panic(err)
			}
		}()

		if err != nil {
			fmt.Printf("Failed to open file: %v\n", err)
			os.Exit(1)
		}

		resetFunction = func(r io.Reader) io.Reader {
			fileReader := r.(*os.File)
			fileReader.Seek(0, io.SeekStart)
			return fileReader
		}
	}

	cliFlags := flag.NFlag()
	var outputFormat func(filename string, values []int)

	switch cliFlags {
	case 0:
		*bytesFlag = true
		*wordsFlag = true
		*linesFlag = true
		outputFormat = func(filename string, values []int) {
			if len(values) != 3 {
				panic("wrong number of values")
			}
			if filename != "" {
				fmt.Printf("    %d   %d  %d %s\n", values[0], values[1], values[2], filename)
			} else {
				fmt.Printf("    %d   %d  %d\n", values[0], values[1], values[2])
			}
		}
	case 1:
		outputFormat = func(filename string, values []int) {
			if len(values) != 1 {
				panic("wrong number of values")
			}
			if filename != "" {
				fmt.Printf("  %d %s\n", values[0], filename)
			} else {
				fmt.Printf("  %d\n", values[0])
			}
		}
	case 2:
		outputFormat = func(filename string, values []int) {
			if len(values) != 2 {
				panic("wrong number of values")
			}
			if filename != "" {
				fmt.Printf("   %d  %d %s\n", values[0], values[1], filename)
			} else {
				fmt.Printf("   %d  %d\n", values[0], values[1])
			}
		}
	case 3:
		outputFormat = func(filename string, values []int) {
			if len(values) != 3 {
				panic("wrong number of values")
			}
			if filename != "" {
				fmt.Printf("    %d   %d  %d %s\n", values[0], values[1], values[2], filename)
			} else {
				fmt.Printf("    %d   %d  %d\n", values[0], values[1], values[2])
			}

		}
	case 4:
		// Means that we've pass bytes and characters.
		// Those two flags cancel each other, so we are going to turn off the character flag
		*characterFlag = false

		outputFormat = func(filename string, values []int) {
			if len(values) != 3 {
				panic("wrong number of values")
			}
			if filename != "" {
				fmt.Printf("    %d   %d  %d %s\n", values[0], values[1], values[2], filename)
			} else {
				fmt.Printf("    %d   %d  %d\n", values[0], values[1], values[2])
			}
		}
	default:
		panic("wrong number of flags")
	}

	lines := 0
	words := 0
	bytes := 0
	characters := 0
	scanner := bufio.NewScanner(reader)

	split := func(data []byte, atEOF bool) (advance int, token []byte, err error) {
		moveForward, result, e := bufio.ScanLines(data, atEOF)

		words += len(strings.Fields(string(result)))

		bytes += moveForward

		return moveForward, result, e
	}
	scanner.Split(split)

	for scanner.Scan() {
		lines++
	}

	if *characterFlag {
		// TODO: Figure out a way to count character with a single loop
		// Theses are the previous attempts
		// characters += len([]rune(string(result)))
		// for _, b := range result {
		// 	characters += len([]rune(string(b)))
		// }
		// 325002

		// characters += utf8.RuneCountInString(string(result))
		// 325002

		// var ia norm.Iter
		// ia.InitString(norm.NFKD, string(result))
		// for !ia.Done() {
		// 	characters++
		// 	ia.Next()
		// }
		// 325077

		// var ia norm.Iter
		// ia.InitString(norm.NFC, string(result))
		// for !ia.Done() {
		// 	characters++
		// 	ia.Next()
		// }
		//325002

		// for _, b := range result {
		// 	characters += len(string(b))
		// }
		// 332456

		// for range string(result) {
		// 	characters++
		// }
		// 325002

		// str := string(data)
		// fmt.Println(str)
		// fmt.Println(len(str))
		// for len(str) > 0 {
		// 	_, size := utf8.DecodeLastRuneInString(str)
		// 	characters += size

		// 	str = str[:len(str)-size]
		// }
		// 327900
		resetedReader := resetFunction(reader)
		scanner = bufio.NewScanner(resetedReader)
		scanner.Split(bufio.ScanRunes)
		for scanner.Scan() {
			characters++
		}
	}

	values := []int{}

	if *linesFlag {
		values = append(values, lines)
	}

	if *wordsFlag {
		values = append(values, words)
	}

	if *characterFlag {
		values = append(values, characters)
	}

	if *bytesFlag {
		values = append(values, bytes)
	}

	outputFormat(fileName, values)
}
