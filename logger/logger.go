package logger

import (
	"log/slog"

	"github.com/ink-splatters/odido-aap/logger/internal"
	"github.com/rs/zerolog"
	slogzerolog "github.com/samber/slog-zerolog/v2"
)

// TODO: read defaults from config

func New() *slog.Logger {
	consoleWriter := internal.NewConsoleLevelWriter()

	
	zerologLogger := slog.New(
		zerolog.MultiLevelWriter(
			internal.NewZerologWriter(),

		)

	)
	// return slog.New(slog.HandlerOptions{Level: slog.LevelDebug})
}