package logger

import (
	"log/slog"
	"github.com/rs/zerolog"
    slogzerolog "github.com/samber/slog-zerolog/v2"
)

func New() *slog.Logger {
	slog.New(slog.HandlerOptions{})
	// return slog.New(slog.HandlerOptions{Level: slog.LevelDebug})
}