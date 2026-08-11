// @prompt 00_nucleo/prompts/core.md
// @layer L1
// @updated 2026-06-08
package core

import "net/http"

func Fetch(url string) {
	http.Get(url)
}
