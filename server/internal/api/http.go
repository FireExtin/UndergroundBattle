package api

import (
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"undergroundbattle/server/pkg/rules"
)

// Purpose: Exposes the in-memory sandbox session over minimal HTTP endpoints and optionally serves the built web debugger.

func NewHandler(session *SandboxSession, staticDir string) http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/api/debugger/messages", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodGet {
			writeMethodNotAllowed(writer, http.MethodGet)
			return
		}

		writeJSON(writer, http.StatusOK, session.Messages())
	})
	mux.HandleFunc("/api/debugger/actions", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodPost {
			writeMethodNotAllowed(writer, http.MethodPost)
			return
		}

		var action rules.Action
		if err := json.NewDecoder(request.Body).Decode(&action); err != nil {
			writeJSON(writer, http.StatusBadRequest, map[string]string{
				"error": "invalid_action_json",
			})
			return
		}

		messages, err := session.SubmitAction(action)
		if err != nil {
			if errors.Is(err, errSetupNotCompleted) {
				writeJSON(writer, http.StatusConflict, map[string]string{
					"error": errSetupNotCompleted.Error(),
				})
				return
			}
			writeJSON(writer, http.StatusInternalServerError, map[string]string{
				"error": err.Error(),
			})
			return
		}

		writeJSON(writer, http.StatusOK, messages)
	})
	mux.HandleFunc("/api/debugger/reset", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodPost {
			writeMethodNotAllowed(writer, http.MethodPost)
			return
		}

		messages, err := session.Reset()
		if err != nil {
			writeJSON(writer, http.StatusInternalServerError, map[string]string{
				"error": err.Error(),
			})
			return
		}

		writeJSON(writer, http.StatusOK, messages)
	})
	mux.HandleFunc("/api/debugger/reports/latest", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodGet {
			writeMethodNotAllowed(writer, http.MethodGet)
			return
		}

		report, ok := session.LatestReport()
		if !ok {
			writeJSON(writer, http.StatusNotFound, map[string]string{
				"error": "report_not_found",
			})
			return
		}

		writeJSON(writer, http.StatusOK, report)
	})
	mux.HandleFunc("/api/debugger/traces/latest", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodGet {
			writeMethodNotAllowed(writer, http.MethodGet)
			return
		}

		trace, ok := session.LatestTrace()
		if !ok {
			writeJSON(writer, http.StatusNotFound, map[string]string{
				"error": "trace_not_found",
			})
			return
		}

		writeJSON(writer, http.StatusOK, trace)
	})
	mux.HandleFunc("/api/battle/setup/state", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodGet {
			writeMethodNotAllowed(writer, http.MethodGet)
			return
		}

		writeJSON(writer, http.StatusOK, session.SetupState())
	})
	mux.HandleFunc("/api/battle/setup/start", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodPost {
			writeMethodNotAllowed(writer, http.MethodPost)
			return
		}

		var input SetupStartInput
		if request.Body != nil {
			if err := json.NewDecoder(request.Body).Decode(&input); err != nil && !errors.Is(err, io.EOF) {
				writeJSON(writer, http.StatusBadRequest, map[string]string{"error": "invalid_setup_start_json"})
				return
			}
		}

		state, err := session.StartSetup(input)
		if err != nil {
			writeJSON(writer, http.StatusInternalServerError, map[string]string{"error": err.Error()})
			return
		}

		writeJSON(writer, http.StatusOK, state)
	})
	mux.HandleFunc("/api/battle/setup/advance", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodPost {
			writeMethodNotAllowed(writer, http.MethodPost)
			return
		}

		var input SetupAdvanceInput
		if request.Body != nil {
			if err := json.NewDecoder(request.Body).Decode(&input); err != nil && !errors.Is(err, io.EOF) {
				writeJSON(writer, http.StatusBadRequest, map[string]string{"error": "invalid_setup_advance_json"})
				return
			}
		}

		state, err := session.AdvanceSetup(input)
		if err != nil {
			status := http.StatusInternalServerError
			if errors.Is(err, errSetupNotActive) {
				status = http.StatusConflict
			}
			writeJSON(writer, status, map[string]string{"error": err.Error()})
			return
		}

		writeJSON(writer, http.StatusOK, state)
	})

	if staticDir == "" || !directoryExists(staticDir) {
		return mux
	}

	fileServer := http.FileServer(http.Dir(staticDir))
	indexPath := filepath.Join(staticDir, "index.html")

	return http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
		if strings.HasPrefix(request.URL.Path, "/api/") {
			mux.ServeHTTP(writer, request)
			return
		}

		if request.URL.Path == "/" || pathWithoutExtension(request.URL.Path) {
			http.ServeFile(writer, request, indexPath)
			return
		}

		fileServer.ServeHTTP(writer, request)
	})
}

func writeJSON(writer http.ResponseWriter, status int, value any) {
	writer.Header().Set("Content-Type", "application/json; charset=utf-8")
	writer.WriteHeader(status)
	if err := json.NewEncoder(writer).Encode(value); err != nil {
		http.Error(writer, err.Error(), http.StatusInternalServerError)
	}
}

func writeMethodNotAllowed(writer http.ResponseWriter, allowed string) {
	writer.Header().Set("Allow", allowed)
	writeJSON(writer, http.StatusMethodNotAllowed, map[string]string{
		"error": "method_not_allowed",
	})
}

func directoryExists(path string) bool {
	info, err := os.Stat(path)
	if err != nil {
		return false
	}

	return info.IsDir()
}

func pathWithoutExtension(path string) bool {
	base := filepath.Base(path)
	return filepath.Ext(base) == ""
}
