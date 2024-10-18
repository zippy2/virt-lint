/* SPDX-License-Identifier: LGPL-3.0-or-later */

package main

import (
	"fmt"
	"io"
	libvirt "libvirt.org/go/libvirt"
	"os"
	"path/filepath"
	"gitlab.com/MichalPrivoznik/virt-lint/go/virt-lint"
)

func list_validator_tags() {
	tags, err := VirtLint.List_validator_tags()

	if err != nil {
		panic(err)
	}

	for _, tag := range tags {
		fmt.Printf("%s\n", tag)
	}
}

func print_help(progname string) {
	base := filepath.Base(progname)
	fmt.Printf(
		`Virtualization linting library

Usage: %s [OPTIONS]

Options:
  -c, --connect <URI>            connection uri
  -p, --path <FILE>              The path to the domain XML, otherwise read the XML from stdin
  -d, --debug                    Turn debugging information on
  -v, --validators <VALIDATORS>  Comma separated list of validator tags, empty means all
  -l, --list-validator-tags      List known validator tags
  -h, --help                     Print help
  -V, --version                  Print version
`,
		base)
}

func print_version() {
	version := VirtLint.Virt_lint_version()

	fmt.Printf("virt-lint: %d.%d.%d\n", version/1000000, version/1000, version%1000)
}

type Cli struct {
	uri   string
	path  *string
	debug bool
	tags  []string
}

func parse_args() Cli {
	var uri string
	var path *string = nil
	var debug = false
	var tags []string

	for i := 1; i < len(os.Args); i++ {
		arg := os.Args[i]
		switch arg {
		case "-c", "--connect":
			if i == len(os.Args)-1 {
				fmt.Printf("%s requires an argument\n", arg)
				os.Exit(1)
			}
			uri = os.Args[i+1]
			i++
		case "-p", "--path":
			if i == len(os.Args)-1 {
				fmt.Printf("%s requires an argument\n", arg)
				os.Exit(1)
			}
			path = &os.Args[i+1]
			i++
		case "-d", "--debug":
			debug = true
		case "-v", "--validators":
			if i == len(os.Args)-1 {
				fmt.Printf("%s requires an argument\n", arg)
				os.Exit(1)
			}
			tags = append(tags, os.Args[i+1])
			i++
		case "-l", "--list-validator-tags":
			list_validator_tags()
			os.Exit(0)
		case "-h", "--help":
			print_help(os.Args[0])
			os.Exit(0)
		case "-V", "--version":
			print_version()
			os.Exit(0)
		default:
			fmt.Printf("Unknown argument: %s\n", arg)
			os.Exit(1)
		}
	}

	return Cli{uri: uri, path: path, debug: debug, tags: tags}
}

func virt_lint_worker(conn *libvirt.Connect, xml string, tags []string) error {
	vl, _ := VirtLint.New(conn)
	defer vl.Close()

	err := vl.Validate(xml, tags, false)
	if err != nil {
		return err
	}

	warn, err := vl.GetWarnings()
	if err != nil {
		return err
	}

	for i := 0; i < len(warn); i++ {
		w := warn[i]
		fmt.Printf("Warning: tags=%v\tdomain=%v\tlevel=%v\tmsg=%s\n", w.Tags, w.Domain, w.Level, w.Msg)
	}

	return nil
}

func main() {
	var domxml string
	cli := parse_args()

	if cli.path != nil {
		data, err := os.ReadFile(*cli.path)
		if err != nil {
			panic(err)
		}
		domxml = string(data)
	} else {
		data, err := io.ReadAll(os.Stdin)
		if err != nil {
			panic(err)
		}
		domxml = string(data)
	}

	conn, err := libvirt.NewConnect(cli.uri)
	if err != nil {
		panic(err)
	}
	defer conn.Close()

	err = virt_lint_worker(conn, domxml, cli.tags)
	if err != nil {
		panic(err)
	}
}
