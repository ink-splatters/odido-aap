package internal

import (
	"os"
	"github.com/rs/zerolog"
)

func NewConsoleWriter() *zerolog.ConsoleWriter {

	consoleWriter := zerolog.NewConsoleWriter(  
		func(w *zerolog.ConsoleWriter) {
		    w.Out = os.Stderr
		},  
	)  

	return &consoleWriter
}

func NewConsoleWriterLeveled(level zerolog.Level) *LevelWriter {
	consoleWriter := NewConsoleWriter()

	consoleWriterLeveled := &LevelWriter{Writer: consoleWriter, Level: zerolog.DebugLevel}  
	return consoleWriterLeveled
}