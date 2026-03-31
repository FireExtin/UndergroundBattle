package main

import (
	"log"
	"net/http"
	"os"
	"path/filepath"
	"runtime"

	"undergroundbattle/server/internal/api"
)

// main starts the minimal sandbox HTTP server, exposing the authoritative rules core and optionally serving the built web debugger.
func main() {
	session := api.NewSandboxSession()
	staticDir := detectWebDistDir()
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	address := ":" + port
	log.Printf("undergroundbattle sandbox listening on %s", address)
	if staticDir != "" {
		log.Printf("serving web debugger from %s", staticDir)
	} else {
		log.Printf("web/dist not found; serving API only")
	}

	if err := http.ListenAndServe(address, api.NewHandler(session, staticDir)); err != nil {
		log.Fatal(err)
	}
}

func detectWebDistDir() string {
	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		return ""
	}

	distDir := filepath.Clean(filepath.Join(filepath.Dir(currentFile), "../../../web/dist"))
	info, err := os.Stat(distDir)
	if err != nil || !info.IsDir() {
		return ""
	}

	return distDir
}
