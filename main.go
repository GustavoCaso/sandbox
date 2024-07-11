package main

import (
	"bufio"
	"flag"
	"fmt"
	"os"
	"strings"
)

var bytesFlag = flag.Bool("c", false, "The number of bytes in each input file is written to the standard output.")
var linesFlag = flag.Bool("l", false, "The number of lines in each input file is written to the standard output.")
var wordsFlag = flag.Bool("w", false, "The number of words in each input file is written to the standard output.")

func main() {
	flag.Parse()

	if len(os.Args) < 2 {
		fmt.Println("You must provide a file")
		os.Exit(1)
	}

	file, err := os.Open(os.Args[2])
	defer file.Close()
	if err != nil {
		fmt.Printf("Failed to read file: %v\n", err)
		os.Exit(1)
	}

	lines := 0
	words := 0
	bytes := 0
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

	if *bytesFlag {
		fmt.Printf("%d %s\n", bytes, os.Args[2])
	}

	if *linesFlag {
		fmt.Printf("%d %s\n", lines, os.Args[2])
	}

	if *wordsFlag {
		fmt.Printf("%d %s\n", words, os.Args[2])
	}
}
