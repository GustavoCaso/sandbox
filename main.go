package main

import (
	"bufio"
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

	argsLength := len(os.Args)

	if argsLength < 2 {
		fmt.Println("You must provide a file")
		os.Exit(1)
	}

	fileName := os.Args[argsLength-1]

	file, err := os.Open(fileName)
	defer file.Close()
	if err != nil {
		fmt.Printf("Failed to read file: %v\n", err)
		os.Exit(1)
	}

	lines := 0
	words := 0
	bytes := 0
	characters := 0
	scanner := bufio.NewScanner(file)

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
		file.Seek(0, io.SeekStart)
		scanner = bufio.NewScanner(file)
		scanner.Split(bufio.ScanRunes)
		for scanner.Scan() {
			characters++
		}
	}

	output := strings.Builder{}

	if *linesFlag {
		fmt.Fprintf(&output, "  %d", lines)
	}

	if *wordsFlag {
		fmt.Fprintf(&output, "  %d", words)
	}

	if *characterFlag {
		fmt.Fprintf(&output, "  %d", characters)
	}

	if *bytesFlag {
		fmt.Fprintf(&output, "  %d", bytes)
	}

	output.WriteString(fmt.Sprintf(" %s", fileName))

	fmt.Println(output.String())
}
