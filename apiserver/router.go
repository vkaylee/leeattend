package main

import (
	"net/http"
	"strings"

	"github.com/go-chi/chi/v5"
)

// registerRoute keeps route declarations compact while preserving net/http handlers.
// It also mirrors Go ServeMux PathValue for existing handlers during chi migration.
func registerRoute(router chi.Router, pattern string, handler http.HandlerFunc) {
	parts := strings.SplitN(pattern, " ", 2)
	if len(parts) != 2 || parts[0] == "" || parts[1] == "" {
		panic("invalid HTTP route pattern: " + pattern)
	}

	method, path := parts[0], parts[1]
	paramNames := routeParamNames(path)
	router.MethodFunc(method, path, func(w http.ResponseWriter, r *http.Request) {
		for _, name := range paramNames {
			r.SetPathValue(name, chi.URLParam(r, name))
		}
		handler(w, r)
	})
}

func routeParamNames(path string) []string {
	var names []string
	for _, segment := range strings.Split(path, "/") {
		if strings.HasPrefix(segment, "{") && strings.HasSuffix(segment, "}") {
			name := strings.TrimSuffix(strings.TrimPrefix(segment, "{"), "}")
			if name != "" {
				names = append(names, name)
			}
		}
	}
	return names
}
